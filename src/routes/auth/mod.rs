use axum::{Router, routing::post};

use crate::state::AppState;

mod blacklist;
mod handlers;
pub mod jwt;
mod kakao;
mod queries;

pub use blacklist::Blacklist;
pub use jwt::JwtKeys;
pub use kakao::KakaoClient;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/kakao/check", post(handlers::check))
        .route("/kakao/sign-up", post(handlers::sign_up))
        .route("/kakao/login", post(handlers::login))
        .route("/reissue", post(handlers::reissue))
        .route("/logout", post(handlers::logout))
}
