use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, Query, State};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::error::KestrelError;
use crate::core::models::User;
use crate::core::types::DbUuid;

/// Wrapper to satisfy the `Box<dyn std::error::Error>` bound on `KestrelError::Internal`
/// for error types (like argon2's) that don't implement `std::error::Error`.
#[derive(Debug)]
struct SimpleError(String);

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SimpleError {}

use crate::core::repository::UserRepository;
use crate::db::pool::DbPool;
use crate::db::postgres::user_repository::PostgresUserRepository;
use crate::db::sqlite::user_repository::SqliteUserRepository;

use super::router::AppState;

// --- JWT Claims ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iss: String,
    pub aud: String,
}

// --- Request / Response types ---

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct TokenRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct LoginParams {
    pub provider: String,
}

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
}

// --- AuthUser extractor (reads from request extensions) ---

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = KestrelError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(KestrelError::Unauthorized)
    }
}

// --- Auth middleware ---

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let token = match extract_bearer_token(&req) {
        Some(t) => t,
        None => return KestrelError::Unauthorized.into_response(),
    };

    match decode_jwt(&token, &state.jwt_secret) {
        Ok(claims) => match Uuid::parse_str(&claims.sub) {
            Ok(user_id) => {
                req.extensions_mut().insert(AuthUser { user_id });
                next.run(req).await
            }
            Err(_) => KestrelError::Unauthorized.into_response(),
        },
        Err(_) => KestrelError::Unauthorized.into_response(),
    }
}

fn extract_bearer_token(req: &Request<Body>) -> Option<String> {
    // 1. Try Cookie header
    if let Some(cookie) = req.headers().get(axum::http::header::COOKIE) {
        if let Ok(cookie_str) = cookie.to_str() {
            for part in cookie_str.split(';') {
                let part = part.trim();
                if let Some(token) = part.strip_prefix("kestrel_token=") {
                    return Some(token.to_string());
                }
            }
        }
    }

    // 2. Fallback to Authorization header
    req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

// --- JWT helpers ---

fn encode_jwt(user_id: &str, secret: &str) -> Result<String, KestrelError> {
    let claims = Claims {
        sub: user_id.to_string(),
        exp: Utc::now().timestamp() + 24 * 60 * 60,
        iss: "kestrel".to_string(),
        aud: "kestrel".to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| KestrelError::Internal(Box::new(e)))
}

fn decode_jwt(token: &str, secret: &str) -> Result<Claims, KestrelError> {
    let mut validation = Validation::default();
    validation.set_audience(&["kestrel"]);
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| KestrelError::Unauthorized)?;
    Ok(data.claims)
}

// --- K-023: POST /api/v1/auth/register ---

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), KestrelError> {
    if body.username.is_empty() || body.password.is_empty() {
        return Err(KestrelError::BadRequest(
            "Username and password are required".to_string(),
        ));
    }

    if body.password.len() < 8 {
        return Err(KestrelError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Password hashing failed: {e}");
            KestrelError::Internal(Box::new(SimpleError(e.to_string())))
        })?
        .to_string();

    let now = Utc::now().timestamp();
    let user = User {
        id: DbUuid::new(Uuid::new_v4()),
        username: body.username,
        password_hash: hash,
        created_at: now,
        updated_at: now,
    };

    create_user(&state.db, &user).await?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id: user.id.to_string(),
        }),
    ))
}

async fn create_user(db: &DbPool, user: &User) -> Result<(), KestrelError> {
    let result = match db {
        DbPool::Sqlite(pool) => SqliteUserRepository::new(pool.clone()).create(user).await,
        DbPool::Postgres(pool) => PostgresUserRepository::new(pool.clone()).create(user).await,
    };

    result.map_err(|e| match e {
        sqlx::Error::Database(e) if is_unique_violation(&*e) => {
            KestrelError::Conflict("Username already exists".to_string())
        }
        sqlx::Error::Database(e) => {
            tracing::warn!("Database error during user creation: {}", e);
            KestrelError::Internal(Box::new(e))
        }
        other => KestrelError::Database(other),
    })
}

fn is_unique_violation(e: &dyn sqlx::error::DatabaseError) -> bool {
    e.message().contains("UNIQUE constraint failed")
        || e.message().contains("duplicate key")
}

// --- K-024: POST /api/v1/auth/token ---

pub async fn token(
    State(state): State<AppState>,
    Json(body): Json<TokenRequest>,
) -> Result<Response, KestrelError> {
    let user = find_user_by_username(&state, &body.username).await?;
    let user = user.ok_or(KestrelError::Unauthorized)?;

    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
        tracing::error!("Invalid stored hash: {e}");
        KestrelError::Internal(Box::new(SimpleError(e.to_string())))
    })?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| KestrelError::Unauthorized)?;

    let jwt = encode_jwt(&user.id.to_string(), &state.jwt_secret)?;

    let cookie = format!("kestrel_token={}; HttpOnly; Path=/; SameSite=Lax", jwt);

    let response = axum::response::Response::builder()
        .header(axum::http::header::SET_COOKIE, cookie)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&TokenResponse {
            token: jwt,
            user_id: user.id.to_string(),
        }).unwrap()))
        .map_err(|e| KestrelError::Internal(Box::new(e)))?;

    Ok(response)
}

// --- GET /api/v1/auth/me ---

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: String,
}

pub async fn me(
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<MeResponse>, KestrelError> {
    Ok(Json(MeResponse {
        user_id: auth_user.user_id.to_string(),
    }))
}

async fn find_user_by_username(
    state: &AppState,
    username: &str,
) -> Result<Option<User>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            Ok(SqliteUserRepository::new(pool.clone())
                .find_by_username(username)
                .await?)
        }
        DbPool::Postgres(pool) => {
            Ok(PostgresUserRepository::new(pool.clone())
                .find_by_username(username)
                .await?)
        }
    }
}

// --- K-026: GET /api/v1/auth/login?provider=x ---

pub async fn login(Query(params): Query<LoginParams>) -> Result<Response, KestrelError> {
    let _base_url = std::env::var("KESTREL_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Future: build OAuth2 authorization URL and redirect
    // let redirect_uri = format!("{}/api/v1/auth/callback", base_url);
    // let auth_url = format!(
    //     "https://{}/oauth/authorize?client_id=...&redirect_uri=...&response_type=code",
    //     params.provider
    // );

    Err(KestrelError::NotImplemented(format!(
        "OAuth login with provider '{}' is not yet implemented. \
         OAuth applications are not registered yet.",
        params.provider
    )))
}

// --- K-027: GET /api/v1/auth/callback ---

pub async fn callback(
    Query(_params): Query<CallbackParams>,
) -> Result<Response, KestrelError> {
    // Future: exchange authorization code for access/refresh tokens,
    // then create or update the linked account.
    Err(KestrelError::NotImplemented(
        "OAuth callback handling is not yet implemented. \
         OAuth applications are not registered yet."
            .to_string(),
    ))
}
