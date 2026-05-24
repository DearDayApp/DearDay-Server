use std::sync::Arc;

use sqlx::PgPool;

use crate::Config;
use crate::routes::auth::{Blacklist, JwtKeys, KakaoClient};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub db: PgPool,
    pub jwt: JwtKeys,
    pub kakao: KakaoClient,
    pub blacklist: Blacklist,
}

impl AppState {
    pub fn new(config: &Config, db: PgPool) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                jwt: JwtKeys::new(&config.jwt_access_secret, &config.jwt_refresh_secret),
                kakao: KakaoClient::new(&config.kakao_api_url),
                blacklist: Blacklist::new(),
            }),
        }
    }
}

impl std::ops::Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
