use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use uuid::Uuid;

use super::calendars::{find_event_from_db, verify_event_ownership};
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::models::EventPollWithVotes;
use crate::core::repository::EventPollRepository;
use crate::db::pool::DbPool;

fn poll_repo(state: &AppState) -> Box<dyn EventPollRepository> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            Box::new(crate::db::sqlite::poll_repository::SqlitePollRepository::new(pool.clone()))
        }
        DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::poll_repository::PostgresPollRepository::new(pool.clone()),
        ),
    }
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreatePollRequest {
    pub question: String,
    pub options: Vec<String>,
}

fn clean_options(options: &[String]) -> Result<Vec<String>, KestrelError> {
    let cleaned: Vec<String> = options
        .iter()
        .map(|o| o.trim().to_string())
        .filter(|o| !o.is_empty())
        .take(8)
        .collect();
    if cleaned.len() < 2 {
        return Err(KestrelError::BadRequest(
            "Poll needs at least 2 non-empty options".to_string(),
        ));
    }
    Ok(cleaned)
}

pub async fn create_poll(
    State(state): State<AppState>,
    user: super::auth::AuthUser,
    Path(event_id): Path<Uuid>,
    Json(body): Json<CreatePollRequest>,
) -> Result<Json<EventPollWithVotes>, KestrelError> {
    let event = find_event_from_db(&state, event_id)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Event not found".to_string()))?;
    verify_event_ownership(&state, user.user_id, &event).await?;

    let question = body.question.trim().to_string();
    if question.is_empty() || question.chars().count() > 280 {
        return Err(KestrelError::BadRequest(
            "Question must be 1-280 characters".to_string(),
        ));
    }
    let options = clean_options(&body.options)?;

    let repo = poll_repo(&state);
    let poll = repo.create_poll(event_id, &question, &options).await?;
    Ok(Json(EventPollWithVotes {
        id: poll.id,
        event_id: poll.event_id,
        question: poll.question,
        options,
        votes: vec![],
    }))
}

pub async fn list_polls(
    State(state): State<AppState>,
    user: super::auth::AuthUser,
    Path(event_id): Path<Uuid>,
) -> Result<Json<Vec<EventPollWithVotes>>, KestrelError> {
    let event = find_event_from_db(&state, event_id)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Event not found".to_string()))?;
    verify_event_ownership(&state, user.user_id, &event).await?;

    Ok(Json(poll_repo(&state).list_polls(event_id).await?))
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct VoteRequest {
    pub voter_email: String,
    pub option_index: i32,
}

pub async fn vote_poll(
    State(state): State<AppState>,
    user: super::auth::AuthUser,
    Path((event_id, poll_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<VoteRequest>,
) -> Result<Json<serde_json::Value>, KestrelError> {
    let event = find_event_from_db(&state, event_id)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Event not found".to_string()))?;
    verify_event_ownership(&state, user.user_id, &event).await?;

    let email = body.voter_email.trim().to_lowercase();
    if !email.contains('@') {
        return Err(KestrelError::BadRequest("Invalid voter email".to_string()));
    }
    if body.option_index < 0 {
        return Err(KestrelError::BadRequest("Invalid option".to_string()));
    }

    let repo = poll_repo(&state);
    let options = repo
        .list_polls(event_id)
        .await?
        .into_iter()
        .find(|p| p.id.0 == poll_id)
        .ok_or_else(|| KestrelError::NotFound("Poll not found".to_string()))?
        .options;
    if body.option_index as usize >= options.len() {
        return Err(KestrelError::BadRequest("Invalid option".to_string()));
    }

    repo.vote(poll_id, &email, body.option_index).await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
