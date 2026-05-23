use axum::extract::FromRef;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Clone)]
pub struct Users {
    db: PgPool,
}

impl FromRef<AppState> for Users {
    fn from_ref(state: &AppState) -> Self {
        Self {
            db: state.db.clone(),
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
}

impl Users {
    pub async fn list(&self) -> sqlx::Result<Vec<User>> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> sqlx::Result<User> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn create(&self, input: CreateUser) -> sqlx::Result<User> {
        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, name)
            VALUES ($1, $2)
            RETURNING id, email, name, created_at, updated_at
            "#,
            input.email,
            input.name,
        )
        .fetch_one(&self.db)
        .await
    }
}
