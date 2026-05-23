use crate::helpers::spawn_app;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn create_user_returns_201(pool: PgPool) {
    let app = spawn_app(pool).await;

    let response = app
        .client
        .post(format!("{}/users", app.address))
        .json(&json!({ "email": "alice@example.com", "name": "Alice" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["email"], "alice@example.com");
    assert_eq!(body["name"], "Alice");
}

#[sqlx::test]
async fn duplicate_email_returns_409(pool: PgPool) {
    let app = spawn_app(pool).await;
    let body = json!({ "email": "dup@example.com", "name": "Dup" });

    app.client
        .post(format!("{}/users", app.address))
        .json(&body)
        .send()
        .await
        .unwrap();

    let response = app
        .client
        .post(format!("{}/users", app.address))
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 409);
}
