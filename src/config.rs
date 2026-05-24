use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub kakao_api_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            jwt_access_secret: std::env::var("JWT_ACCESS_SECRET")
                .context("JWT_ACCESS_SECRET must be set")?,
            jwt_refresh_secret: std::env::var("JWT_REFRESH_SECRET")
                .context("JWT_REFRESH_SECRET must be set")?,
            kakao_api_url: std::env::var("KAKAO_API_URL")
                .unwrap_or_else(|_| "https://kapi.kakao.com".into()),
        })
    }
}
