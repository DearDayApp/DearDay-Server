use axum::extract::FromRef;
use reqwest::Client;
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

#[derive(Clone)]
pub struct KakaoClient {
    http: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct KakaoUser {
    pub id: i64,
    pub kakao_account: Option<KakaoAccount>,
}

#[derive(Debug, Deserialize)]
pub struct KakaoAccount {
    pub profile: Option<KakaoProfile>,
}

#[derive(Debug, Deserialize)]
pub struct KakaoProfile {
    pub nickname: Option<String>,
}

impl KakaoClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get_user_info(&self, access_token: &str) -> ApiResult<KakaoUser> {
        let url = format!("{}/v2/user/me", self.base_url);
        let response = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ApiError::Unauthorized("invalid kakao access token".into()));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Internal(anyhow::anyhow!(
                "kakao /v2/user/me returned {status}: {body}"
            )));
        }

        response
            .json::<KakaoUser>()
            .await
            .map_err(|e| ApiError::Internal(e.into()))
    }
}

impl FromRef<AppState> for KakaoClient {
    fn from_ref(state: &AppState) -> Self {
        state.kakao.clone()
    }
}
