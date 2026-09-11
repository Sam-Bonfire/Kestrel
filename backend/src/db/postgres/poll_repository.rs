use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::{EventPoll, EventPollWithVotes, PollVote};
use crate::core::repository::EventPollRepository;
use crate::core::types::DbUuid;

pub struct PostgresPollRepository {
    pool: PgPool,
}

impl PostgresPollRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventPollRepository for PostgresPollRepository {
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
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(poll.id.0)
        .bind(poll.event_id.0)
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
             FROM event_polls WHERE event_id = $1 ORDER BY created_at",
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(polls.len());
        for poll in polls {
            let votes = sqlx::query_as::<_, PollVote>(
                "SELECT poll_id, voter_email, option_index, created_at
                 FROM poll_votes WHERE poll_id = $1",
            )
            .bind(poll.id.0)
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
        let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM event_polls WHERE id = $1")
            .bind(poll_id)
            .fetch_optional(&self.pool)
            .await?;
        if exists.is_none() {
            return Ok(false);
        }
        sqlx::query(
            "INSERT INTO poll_votes (poll_id, voter_email, option_index) VALUES ($1, $2, $3)
             ON CONFLICT (poll_id, voter_email, option_index) DO NOTHING",
        )
        .bind(poll_id)
        .bind(voter_email)
        .bind(option_index)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }
}
