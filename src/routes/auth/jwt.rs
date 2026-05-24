use axum::extract::FromRef;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub const ACCESS_TTL: Duration = Duration::minutes(30);
pub const REFRESH_TTL: Duration = Duration::days(14);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub jti: Uuid,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JwtKeys {
    access_encode: EncodingKey,
    access_decode: DecodingKey,
    refresh_encode: EncodingKey,
    refresh_decode: DecodingKey,
}

impl JwtKeys {
    pub fn new(access_secret: &str, refresh_secret: &str) -> Self {
        Self {
            access_encode: EncodingKey::from_secret(access_secret.as_bytes()),
            access_decode: DecodingKey::from_secret(access_secret.as_bytes()),
            refresh_encode: EncodingKey::from_secret(refresh_secret.as_bytes()),
            refresh_decode: DecodingKey::from_secret(refresh_secret.as_bytes()),
        }
    }

    pub fn issue(&self, user_id: Uuid, token_type: TokenType) -> ApiResult<IssuedToken> {
        let now = Utc::now();
        let ttl = match token_type {
            TokenType::Access => ACCESS_TTL,
            TokenType::Refresh => REFRESH_TTL,
        };
        let exp = now + ttl;
        let jti = Uuid::new_v4();
        let claims = Claims {
            sub: user_id,
            jti,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };
        let key = match token_type {
            TokenType::Access => &self.access_encode,
            TokenType::Refresh => &self.refresh_encode,
        };
        let token = encode(&Header::new(Algorithm::HS256), &claims, key)
            .map_err(|e| ApiError::Internal(e.into()))?;
        Ok(IssuedToken { token, jti, exp })
    }

    pub fn verify(&self, token: &str, token_type: TokenType) -> ApiResult<Claims> {
        let key = match token_type {
            TokenType::Access => &self.access_decode,
            TokenType::Refresh => &self.refresh_decode,
        };
        let validation = Validation::new(Algorithm::HS256);
        decode::<Claims>(token, key, &validation)
            .map(|data| data.claims)
            .map_err(|_| ApiError::Unauthorized("invalid token".into()))
    }
}

impl FromRef<AppState> for JwtKeys {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}

pub struct IssuedToken {
    pub token: String,
    pub jti: Uuid,
    pub exp: DateTime<Utc>,
}
