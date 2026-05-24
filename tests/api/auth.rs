use serde_json::{Value, json};
use sqlx::PgPool;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{header, method, path},
};

use crate::helpers::{TestApp, spawn_app};

async fn mock_kakao_user(app: &TestApp, kakao_id: i64) {
    Mock::given(method("GET"))
        .and(path("/v2/user/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": kakao_id,
            "kakao_account": { "profile": { "nickname": "테스트유저" } }
        })))
        .mount(&app.kakao_mock)
        .await;
}

async fn mock_kakao_unauthorized(app: &TestApp) {
    Mock::given(method("GET"))
        .and(path("/v2/user/me"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&app.kakao_mock)
        .await;
}

async fn sign_up(app: &TestApp, kakao_id: i64, name: &str) -> Value {
    mock_kakao_user(app, kakao_id).await;
    let response = app
        .client
        .post(format!("{}/auth/kakao/sign-up", app.address))
        .bearer_auth("fake-kakao-token")
        .json(&json!({ "name": name }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);
    response.json().await.unwrap()
}

#[sqlx::test]
async fn check_returns_false_for_unregistered_user(pool: PgPool) {
    let app = spawn_app(pool).await;
    mock_kakao_user(&app, 1001).await;

    let response = app
        .client
        .post(format!("{}/auth/kakao/check", app.address))
        .bearer_auth("fake-kakao-token")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["registered"], false);
}

#[sqlx::test]
async fn check_returns_true_after_signup(pool: PgPool) {
    let app = spawn_app(pool).await;
    sign_up(&app, 1002, "다리아").await;

    let response = app
        .client
        .post(format!("{}/auth/kakao/check", app.address))
        .bearer_auth("fake-kakao-token")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["registered"], true);
}

#[sqlx::test]
async fn sign_up_creates_user_and_returns_tokens(pool: PgPool) {
    let app = spawn_app(pool).await;
    let body = sign_up(&app, 1003, "다리아").await;

    assert!(body["user_id"].as_str().is_some());
    assert!(body["access_token"].as_str().unwrap().split('.').count() == 3);
    assert!(body["refresh_token"].as_str().unwrap().split('.').count() == 3);
}

#[sqlx::test]
async fn sign_up_twice_returns_conflict(pool: PgPool) {
    let app = spawn_app(pool).await;
    sign_up(&app, 1004, "다리아").await;

    let response = app
        .client
        .post(format!("{}/auth/kakao/sign-up", app.address))
        .bearer_auth("fake-kakao-token")
        .json(&json!({ "name": "다리아" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 409);
}

#[sqlx::test]
async fn login_without_signup_returns_404(pool: PgPool) {
    let app = spawn_app(pool).await;
    mock_kakao_user(&app, 1005).await;

    let response = app
        .client
        .post(format!("{}/auth/kakao/login", app.address))
        .bearer_auth("fake-kakao-token")
        .json(&json!({}))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[sqlx::test]
async fn login_after_signup_returns_tokens(pool: PgPool) {
    let app = spawn_app(pool).await;
    let signup_body = sign_up(&app, 1006, "다리아").await;

    let response = app
        .client
        .post(format!("{}/auth/kakao/login", app.address))
        .bearer_auth("fake-kakao-token")
        .json(&json!({ "fcm_token": "new-fcm-token" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["user_id"], signup_body["user_id"]);
    // New login issues fresh tokens (different from sign-up's)
    assert_ne!(body["access_token"], signup_body["access_token"]);
    assert_ne!(body["refresh_token"], signup_body["refresh_token"]);
}

#[sqlx::test]
async fn login_with_invalid_kakao_token_returns_401(pool: PgPool) {
    let app = spawn_app(pool).await;
    mock_kakao_unauthorized(&app).await;

    let response = app
        .client
        .post(format!("{}/auth/kakao/login", app.address))
        .bearer_auth("bad-kakao-token")
        .json(&json!({}))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);
}

#[sqlx::test]
async fn reissue_returns_new_pair(pool: PgPool) {
    let app = spawn_app(pool).await;
    let signup_body = sign_up(&app, 1007, "다리아").await;
    let refresh = signup_body["refresh_token"].as_str().unwrap();

    let response = app
        .client
        .post(format!("{}/auth/reissue", app.address))
        .bearer_auth(refresh)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_ne!(body["refresh_token"], signup_body["refresh_token"]);
}

#[sqlx::test]
async fn reissuing_with_old_refresh_after_reissue_fails(pool: PgPool) {
    let app = spawn_app(pool).await;
    let signup_body = sign_up(&app, 1008, "다리아").await;
    let first_refresh = signup_body["refresh_token"].as_str().unwrap().to_string();

    let reissue1 = app
        .client
        .post(format!("{}/auth/reissue", app.address))
        .bearer_auth(&first_refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(reissue1.status(), 200);

    let reissue_again = app
        .client
        .post(format!("{}/auth/reissue", app.address))
        .bearer_auth(&first_refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(reissue_again.status(), 401);
}

#[sqlx::test]
async fn login_invalidates_previous_refresh_single_session(pool: PgPool) {
    let app = spawn_app(pool).await;
    let first_session = sign_up(&app, 1009, "다리아").await;
    let first_refresh = first_session["refresh_token"].as_str().unwrap().to_string();

    let second_login = app
        .client
        .post(format!("{}/auth/kakao/login", app.address))
        .bearer_auth("fake-kakao-token")
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(second_login.status(), 200);

    let try_reissue_old = app
        .client
        .post(format!("{}/auth/reissue", app.address))
        .bearer_auth(&first_refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(try_reissue_old.status(), 401);
}

#[sqlx::test]
async fn logout_revokes_refresh(pool: PgPool) {
    let app = spawn_app(pool).await;
    let signup_body = sign_up(&app, 1010, "다리아").await;
    let refresh = signup_body["refresh_token"].as_str().unwrap().to_string();

    let logout = app
        .client
        .post(format!("{}/auth/logout", app.address))
        .bearer_auth(&refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(logout.status(), 204);

    let try_reissue = app
        .client
        .post(format!("{}/auth/reissue", app.address))
        .bearer_auth(&refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(try_reissue.status(), 401);
}

#[sqlx::test]
async fn missing_authorization_header_returns_401(pool: PgPool) {
    let app = spawn_app(pool).await;

    let response = app
        .client
        .post(format!("{}/auth/kakao/check", app.address))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

#[sqlx::test]
async fn kakao_is_called_with_provided_bearer_token(pool: PgPool) {
    let app = spawn_app(pool).await;

    Mock::given(method("GET"))
        .and(path("/v2/user/me"))
        .and(header("authorization", "Bearer the-real-kakao-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": 2001i64 })))
        .mount(&app.kakao_mock)
        .await;

    let response = app
        .client
        .post(format!("{}/auth/kakao/check", app.address))
        .bearer_auth("the-real-kakao-token")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}
