use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use super::queries::{CreateUser, User, Users};
use crate::error::ApiResult;

pub(super) async fn list(State(users): State<Users>) -> ApiResult<Json<Vec<User>>> {
    Ok(Json(users.list().await?))
}

pub(super) async fn get_one(
    State(users): State<Users>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<User>> {
    Ok(Json(users.find_by_id(id).await?))
}

pub(super) async fn create(
    State(users): State<Users>,
    Json(input): Json<CreateUser>,
) -> ApiResult<(StatusCode, Json<User>)> {
    let user = users.create(input).await?;
    Ok((StatusCode::CREATED, Json(user)))
}
