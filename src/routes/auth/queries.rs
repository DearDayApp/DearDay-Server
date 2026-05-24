use axum::extract::FromRef;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::state::AppState;

pub const PROVIDER_KAKAO: &str = "KAKAO";

#[derive(Clone)]
pub struct AuthDb {
    db: PgPool,
}

impl FromRef<AppState> for AuthDb {
    fn from_ref(state: &AppState) -> Self {
        Self {
            db: state.db.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserAuth {
    pub user_id: Uuid,
    pub is_deleted: bool,
    pub current_refresh_jti: Option<Uuid>,
}

impl AuthDb {
    pub async fn find_user_by_provider(
        &self,
        provider: &str,
        provider_id: &str,
    ) -> sqlx::Result<Option<UserAuth>> {
        sqlx::query_as!(
            UserAuth,
            r#"
            SELECT u.id AS user_id, u.is_deleted, u.current_refresh_jti
            FROM user_providers up
            JOIN users u ON u.id = up.user_id
            WHERE up.provider = $1 AND up.provider_id = $2
            "#,
            provider,
            provider_id,
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn create_user_with_provider(
        &self,
        name: &str,
        fcm_token: Option<&str>,
        provider: &str,
        provider_id: &str,
    ) -> sqlx::Result<Uuid> {
        let mut tx = self.db.begin().await?;
        let row = sqlx::query!(
            r#"
            INSERT INTO users (name, fcm_token)
            VALUES ($1, $2)
            RETURNING id
            "#,
            name,
            fcm_token,
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO user_providers (user_id, provider, provider_id)
            VALUES ($1, $2, $3)
            "#,
            row.id,
            provider,
            provider_id,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.id)
    }

    pub async fn update_fcm_token(
        &self,
        user_id: Uuid,
        fcm_token: Option<&str>,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE users SET fcm_token = $1, updated_at = NOW() WHERE id = $2",
            fcm_token,
            user_id,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn set_current_refresh_jti(&self, user_id: Uuid, jti: Uuid) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE users SET current_refresh_jti = $1, updated_at = NOW() WHERE id = $2",
            jti,
            user_id,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn clear_current_refresh_jti(&self, user_id: Uuid) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE users SET current_refresh_jti = NULL, updated_at = NOW() WHERE id = $1",
            user_id,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
