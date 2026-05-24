use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

use super::blacklist::Blacklist;
use super::jwt::{JwtKeys, TokenType};
use super::kakao::KakaoClient;
use super::queries::{AuthDb, PROVIDER_KAKAO, UserAuth};

fn extract_bearer(headers: &HeaderMap) -> ApiResult<&str> {
    let header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("missing Authorization header".into()))?;
    header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Authorization must use Bearer scheme".into()))
}

#[derive(Serialize)]
pub struct CheckResponse {
    pub registered: bool,
}

pub(super) async fn check(
    State(kakao): State<KakaoClient>,
    State(db): State<AuthDb>,
    headers: HeaderMap,
) -> ApiResult<Json<CheckResponse>> {
    let kakao_token = extract_bearer(&headers)?;
    let kakao_user = kakao.get_user_info(kakao_token).await?;
    let registered = db
        .find_user_by_provider(PROVIDER_KAKAO, &kakao_user.id.to_string())
        .await?
        .is_some();
    Ok(Json(CheckResponse { registered }))
}

#[derive(Deserialize)]
pub struct SignUpInput {
    pub name: String,
    pub fcm_token: Option<String>,
}

#[derive(Serialize)]
pub struct TokenPair {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
}

pub(super) async fn sign_up(
    State(kakao): State<KakaoClient>,
    State(db): State<AuthDb>,
    State(jwt): State<JwtKeys>,
    State(blacklist): State<Blacklist>,
    headers: HeaderMap,
    Json(input): Json<SignUpInput>,
) -> ApiResult<(StatusCode, Json<TokenPair>)> {
    let kakao_token = extract_bearer(&headers)?;
    let kakao_user = kakao.get_user_info(kakao_token).await?;
    let provider_id = kakao_user.id.to_string();

    if db
        .find_user_by_provider(PROVIDER_KAKAO, &provider_id)
        .await?
        .is_some()
    {
        return Err(ApiError::Conflict("user already registered".into()));
    }

    let user_id = db
        .create_user_with_provider(
            &input.name,
            input.fcm_token.as_deref(),
            PROVIDER_KAKAO,
            &provider_id,
        )
        .await?;

    let pair = issue_and_rotate(&jwt, &db, &blacklist, user_id, None).await?;
    Ok((StatusCode::CREATED, Json(pair)))
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub fcm_token: Option<String>,
}

pub(super) async fn login(
    State(kakao): State<KakaoClient>,
    State(db): State<AuthDb>,
    State(jwt): State<JwtKeys>,
    State(blacklist): State<Blacklist>,
    headers: HeaderMap,
    Json(input): Json<LoginInput>,
) -> ApiResult<Json<TokenPair>> {
    let kakao_token = extract_bearer(&headers)?;
    let kakao_user = kakao.get_user_info(kakao_token).await?;
    let provider_id = kakao_user.id.to_string();

    let UserAuth {
        user_id,
        is_deleted,
        current_refresh_jti,
    } = db
        .find_user_by_provider(PROVIDER_KAKAO, &provider_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("user not registered".into()))?;

    if is_deleted {
        return Err(ApiError::Unauthorized("account deleted".into()));
    }

    if input.fcm_token.is_some() {
        db.update_fcm_token(user_id, input.fcm_token.as_deref())
            .await?;
    }

    let pair = issue_and_rotate(&jwt, &db, &blacklist, user_id, current_refresh_jti).await?;
    Ok(Json(pair))
}

pub(super) async fn reissue(
    State(jwt): State<JwtKeys>,
    State(db): State<AuthDb>,
    State(blacklist): State<Blacklist>,
    headers: HeaderMap,
) -> ApiResult<Json<TokenPair>> {
    let refresh_token = extract_bearer(&headers)?;
    let claims = jwt.verify(refresh_token, TokenType::Refresh)?;
    if blacklist.is_revoked(claims.jti).await {
        return Err(ApiError::Unauthorized("refresh token revoked".into()));
    }

    let pair = issue_and_rotate(&jwt, &db, &blacklist, claims.sub, Some(claims.jti)).await?;
    Ok(Json(pair))
}

pub(super) async fn logout(
    State(jwt): State<JwtKeys>,
    State(db): State<AuthDb>,
    State(blacklist): State<Blacklist>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let refresh_token = extract_bearer(&headers)?;
    let claims = jwt.verify(refresh_token, TokenType::Refresh)?;
    blacklist.revoke(claims.jti).await;
    db.clear_current_refresh_jti(claims.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Blacklist the previous refresh JTI (if any), issue a fresh access+refresh pair,
/// and persist the new refresh JTI as the user's active session.
async fn issue_and_rotate(
    jwt: &JwtKeys,
    db: &AuthDb,
    blacklist: &Blacklist,
    user_id: Uuid,
    previous_refresh_jti: Option<Uuid>,
) -> ApiResult<TokenPair> {
    if let Some(old_jti) = previous_refresh_jti {
        blacklist.revoke(old_jti).await;
    }
    let access = jwt.issue(user_id, TokenType::Access)?;
    let refresh = jwt.issue(user_id, TokenType::Refresh)?;
    db.set_current_refresh_jti(user_id, refresh.jti).await?;
    Ok(TokenPair {
        user_id,
        access_token: access.token,
        refresh_token: refresh.token,
    })
}
