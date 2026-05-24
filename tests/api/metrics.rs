use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test]
async fn metrics_endpoint_exposes_users_active_gauge(pool: PgPool) {
    let app = spawn_app(pool).await;

    let response = app
        .client
        .get(format!("{}/metrics", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = response.text().await.unwrap();
    assert!(
        body.contains("dearday_users_active"),
        "expected dearday_users_active in /metrics output, got:\n{body}"
    );
}
