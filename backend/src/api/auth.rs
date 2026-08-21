use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use async_trait::async_trait;
use axum::Json;
use axum::body::Body;
use axum::extract::{FromRequestParts, Query, State};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::error::KestrelError;
use crate::core::models::User;
use crate::core::types::DbUuid;

use crate::core::error::SimpleError;

use crate::core::repository::{AccountRepository, UserRepository};
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
    if let Some(cookie) = req.headers().get(axum::http::header::COOKIE)
        && let Ok(cookie_str) = cookie.to_str()
    {
        for part in cookie_str.split(';') {
            let part = part.trim();
            if let Some(token) = part.strip_prefix("kestrel_token=") {
                return Some(token.to_string());
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
    e.message().contains("UNIQUE constraint failed") || e.message().contains("duplicate key")
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
        .body(axum::body::Body::from(
            serde_json::to_string(&TokenResponse {
                token: jwt,
                user_id: user.id.to_string(),
            })
            .unwrap(),
        ))
        .map_err(|e| KestrelError::Internal(Box::new(e)))?;

    Ok(response)
}

// --- GET /api/v1/auth/me ---

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: String,
}

pub async fn me(auth_user: axum::Extension<AuthUser>) -> Result<Json<MeResponse>, KestrelError> {
    Ok(Json(MeResponse {
        user_id: auth_user.user_id.to_string(),
    }))
}

async fn find_user_by_username(
    state: &AppState,
    username: &str,
) -> Result<Option<User>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => Ok(SqliteUserRepository::new(pool.clone())
            .find_by_username(username)
            .await?),
        DbPool::Postgres(pool) => Ok(PostgresUserRepository::new(pool.clone())
            .find_by_username(username)
            .await?),
    }
}

// --- K-026: GET /api/v1/auth/login?provider=x ---

pub async fn login(Query(params): Query<LoginParams>) -> Result<Response, KestrelError> {
    let base_url =
        std::env::var("KESTREL_BASE_URL").unwrap_or_else(|_| "http://localhost:1420".to_string());

    let auth_url = match params.provider.as_str() {
        "gmail" => {
            let client_id = std::env::var("GMAIL_CLIENT_ID").unwrap_or_default();
            if client_id.is_empty() {
                return Err(KestrelError::Internal(Box::new(SimpleError(
                    "GMAIL_CLIENT_ID not set".to_string(),
                ))));
            }
            let redirect_uri = format!("{}/api/v1/auth/callback/gmail", base_url);
            let scopes = "https://mail.google.com/ https://www.googleapis.com/auth/calendar";
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent",
                client_id,
                urlencoding::encode(&redirect_uri),
                urlencoding::encode(scopes)
            )
        }
        "outlook" => {
            let client_id = std::env::var("OUTLOOK_CLIENT_ID").unwrap_or_default();
            if client_id.is_empty() {
                return Err(KestrelError::Internal(Box::new(SimpleError(
                    "OUTLOOK_CLIENT_ID not set".to_string(),
                ))));
            }
            let redirect_uri = format!("{}/api/v1/auth/callback/outlook", base_url);
            let scopes = "offline_access Mail.ReadWrite Mail.Send Calendars.ReadWrite";
            format!(
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
                client_id,
                urlencoding::encode(&redirect_uri),
                urlencoding::encode(scopes)
            )
        }
        _ => return Err(KestrelError::BadRequest("Unknown provider".to_string())),
    };

    Ok(axum::response::Redirect::to(&auth_url).into_response())
}

// --- K-027: GET /api/v1/auth/callback/:provider ---

pub async fn callback(
    State(state): State<AppState>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
    axum::extract::Path(provider): axum::extract::Path<String>,
    Query(params): Query<CallbackParams>,
) -> Result<Response, KestrelError> {
    let code = params
        .code
        .ok_or_else(|| KestrelError::BadRequest("Missing code".to_string()))?;

    tracing::info!(
        "Received OAuth callback for {} with code: {}",
        provider,
        code
    );

    let base_url =
        std::env::var("KESTREL_BASE_URL").unwrap_or_else(|_| "http://localhost:1420".to_string());
    let redirect_uri = format!("{}/api/v1/auth/callback/{}", base_url, provider);

    let (token_url, client_id, client_secret) = match provider.as_str() {
        "gmail" => (
            "https://oauth2.googleapis.com/token".to_string(),
            std::env::var("GMAIL_CLIENT_ID").unwrap_or_default(),
            std::env::var("GMAIL_CLIENT_SECRET").unwrap_or_default(),
        ),
        "outlook" => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
            std::env::var("OUTLOOK_CLIENT_ID").unwrap_or_default(),
            std::env::var("OUTLOOK_CLIENT_SECRET").unwrap_or_default(),
        ),
        _ => return Err(KestrelError::BadRequest("Unknown provider".to_string())),
    };

    if client_id.is_empty() || client_secret.is_empty() {
        return Err(KestrelError::Internal(Box::new(SimpleError(format!(
            "Missing credentials for {}",
            provider
        )))));
    }

    let client = reqwest::Client::new();
    let res = client
        .post(&token_url)
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| KestrelError::Internal(Box::new(e)))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        tracing::error!("OAuth token exchange failed: {}", err_text);
        return Err(KestrelError::Internal(Box::new(SimpleError(
            "OAuth token exchange failed".to_string(),
        ))));
    }

    let token_data: serde_json::Value = res
        .json()
        .await
        .map_err(|e| KestrelError::Internal(Box::new(e)))?;

    let access_token = token_data["access_token"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let refresh_token = token_data["refresh_token"].as_str().map(|s| s.to_string());
    let expires_in = token_data["expires_in"].as_i64().unwrap_or(3600);

    let token_expires_at = chrono::Utc::now().timestamp() + expires_in;

    // Fetch User Profile
    let (profile_url, id_field, name_field, email_field) = match provider.as_str() {
        "gmail" => (
            "https://www.googleapis.com/oauth2/v2/userinfo",
            "id",
            "name",
            "email",
        ),
        "outlook" => (
            "https://graph.microsoft.com/v1.0/me",
            "id",
            "displayName",
            "mail", // or userPrincipalName
        ),
        _ => {
            return Err(KestrelError::Internal(Box::new(SimpleError(
                "Unknown provider".to_string(),
            ))));
        }
    };

    let profile_res = client
        .get(profile_url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| KestrelError::Internal(Box::new(e)))?;

    if !profile_res.status().is_success() {
        let err_text = profile_res.text().await.unwrap_or_default();
        tracing::error!("OAuth profile fetch failed: {}", err_text);
        return Err(KestrelError::Internal(Box::new(SimpleError(
            "OAuth profile fetch failed".to_string(),
        ))));
    }

    let profile_data: serde_json::Value = profile_res
        .json()
        .await
        .map_err(|e| KestrelError::Internal(Box::new(e)))?;
    let provider_account_id = profile_data[id_field]
        .as_str()
        .unwrap_or(&format!("{}-{}", provider, Uuid::new_v4()))
        .to_string();
    let display_name = profile_data[name_field]
        .as_str()
        .unwrap_or(&format!("{} Account", provider))
        .to_string();

    // Outlook sometimes uses userPrincipalName if mail is null
    let email = if provider == "outlook" {
        profile_data["mail"]
            .as_str()
            .or(profile_data["userPrincipalName"].as_str())
            .unwrap_or("")
            .to_string()
    } else {
        profile_data[email_field].as_str().unwrap_or("").to_string()
    };

    let final_display_name = if !email.is_empty() {
        email.clone()
    } else {
        display_name
    };

    // Save to database
    let now = chrono::Utc::now().timestamp();
    let account = crate::core::models::Account {
        id: crate::core::types::DbUuid::new(Uuid::new_v4()),
        user_id: crate::core::types::DbUuid::new(auth_user.user_id),
        provider: provider.clone(),
        provider_account_id,
        display_name: final_display_name,
        access_token: Some(access_token.clone()),
        refresh_token,
        token_expires_at: Some(token_expires_at),
        sync_error: None,
        created_at: now,
        updated_at: now,
    };

    let db_res = match &state.db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(pool.clone())
                .create(&account)
                .await
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(pool.clone())
                .create(&account)
                .await
        }
    };

    if let Err(e) = db_res {
        tracing::error!("Failed to save account to DB: {:?}", e);
        return Err(KestrelError::Internal(Box::new(SimpleError(
            "Failed to save account".to_string(),
        ))));
    }

    // Trigger initial historical sync in the background
    let sync_state = state.clone();
    let sync_account = account.clone();
    let sync_token = access_token.clone();
    tokio::spawn(async move {
        tracing::info!(
            "Starting initial historical sync for account {}",
            sync_account.id.0
        );
        if let Err(e) =
            crate::api::sync::sync_account_messages(&sync_state, &sync_account, &sync_token).await
        {
            tracing::error!(
                "Initial historical sync failed for {}: {}",
                sync_account.id.0,
                e
            );
        }
        tracing::info!(
            "Completed initial historical sync for account {}",
            sync_account.id.0
        );
    });

    let frontend_url = std::env::var("KESTREL_FRONTEND_URL")
        .unwrap_or_else(|_| "kestrel://oauth/callback".to_string());

    // Redirect back to the app via deep link
    Ok(axum::response::Redirect::to(&frontend_url).into_response())
}
