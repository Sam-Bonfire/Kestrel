use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::{EventPoll, EventPollWithVotes, PollVote};
use crate::core::repository::EventPollRepository;
use crate::core::types::DbUuid;

pub struct SqlitePollRepository {
    pool: SqlitePool,
}

impl SqlitePollRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventPollRepository for SqlitePollRepository {
    async fn create_poll(
        &self,
        event_id: Uuid,
        question: &str,
        options: &[String],
    ) -> Result<EventPoll, sqlx::Error> {
        let poll = EventPoll {
            id: DbUuid::from(Uuid::new_v4()),
            event_id: DbUuid::from(event_id),
            question: question.to_string(),
            options_json: serde_json::to_string(options).unwrap_or_default(),
            created_at: chrono::Utc::now().timestamp(),
        };
        sqlx::query(
            "INSERT INTO event_polls (id, event_id, question, options_json, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(poll.id.to_string())
        .bind(poll.event_id.to_string())
        .bind(&poll.question)
        .bind(&poll.options_json)
        .bind(poll.created_at)
        .execute(&self.pool)
        .await?;

        Ok(poll)
    }

    async fn list_polls(&self, event_id: Uuid) -> Result<Vec<EventPollWithVotes>, sqlx::Error> {
        let polls = sqlx::query_as::<_, EventPoll>(
            "SELECT id, event_id, question, options_json, created_at
             FROM event_polls WHERE event_id = ? ORDER BY created_at",
        )
        .bind(event_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(polls.len());
        for poll in polls {
            let votes = sqlx::query_as::<_, PollVote>(
                "SELECT poll_id, voter_email, option_index, created_at
                 FROM poll_votes WHERE poll_id = ?",
            )
            .bind(poll.id.to_string())
            .fetch_all(&self.pool)
            .await?;
            let options: Vec<String> = serde_json::from_str(&poll.options_json).unwrap_or_default();
            out.push(EventPollWithVotes {
                id: poll.id,
                event_id: poll.event_id,
                question: poll.question,
                options,
                votes,
            });
        }
        Ok(out)
    }

    async fn vote(
        &self,
        poll_id: Uuid,
        voter_email: &str,
        option_index: i32,
    ) -> Result<bool, sqlx::Error> {
        let exists: Option<String> = sqlx::query_scalar("SELECT id FROM event_polls WHERE id = ?")
            .bind(poll_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        if exists.is_none() {
            return Ok(false);
        }
        sqlx::query(
            "INSERT INTO poll_votes (poll_id, voter_email, option_index) VALUES (?, ?, ?)
             ON CONFLICT(poll_id, voter_email, option_index) DO NOTHING",
        )
        .bind(poll_id.to_string())
        .bind(voter_email)
        .bind(option_index)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }
}
