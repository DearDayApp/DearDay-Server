//! End-to-end tests against the real Kakao API.
//!
//! Marked `#[ignore]` so they don't run in CI by default. Run with:
//!
//! ```
//! KAKAO_TEST_ACCESS_TOKEN=<token> cargo test --test api real_kakao -- --ignored --nocapture
//! ```
//!
//! See `docs/agents/kakao-auth-testing.md` for how to obtain a token.

use dearday::{AppState, Config, run};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::net::TcpListener;

const KAKAO_API_URL: &str = "https://kapi.kakao.com";

fn kakao_access_token() -> Option<String> {
    std::env::var("KAKAO_TEST_ACCESS_TOKEN").ok()
}

async fn spawn_with_real_kakao(db: PgPool) -> String {
    let config = Config {
        bind_addr: "127.0.0.1:0".into(),
        database_url: "unused-in-tests".into(),
        jwt_access_secret: "test-access-secret-must-be-long-enough".into(),
        jwt_refresh_secret: "test-refresh-secret-must-be-long-enough".into(),
        kakao_api_url: KAKAO_API_URL.into(),
    };
    let state = AppState::new(&config, db);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(run(listener, state));
    format!("http://127.0.0.1:{port}")
}

#[sqlx::test]
#[ignore = "requires KAKAO_TEST_ACCESS_TOKEN env var"]
async fn real_kakao_full_flow(pool: PgPool) {
    let Some(token) = kakao_access_token() else {
        eprintln!(
            "\nskipping: KAKAO_TEST_ACCESS_TOKEN not set — see docs/agents/kakao-auth-testing.md\n"
        );
        return;
    };

    let address = spawn_with_real_kakao(pool).await;
    let client = reqwest::Client::new();

    // 1. /check — fresh DB, expect false
    let check_response = client
        .post(format!("{address}/auth/kakao/check"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(check_response.status(), 200, "check returned non-200");
    let check_body: Value = check_response.json().await.unwrap();
    println!("[check #1] {check_body}");
    assert_eq!(check_body["registered"], false);

    // 2. /sign-up — creates the user, returns tokens
    let signup_response = client
        .post(format!("{address}/auth/kakao/sign-up"))
        .bearer_auth(&token)
        .json(&json!({ "name": "RealKakaoTester", "fcm_token": "test-fcm-123" }))
        .send()
        .await
        .unwrap();
    assert_eq!(signup_response.status(), 201, "sign-up returned non-201");
    let signup_body: Value = signup_response.json().await.unwrap();
    println!("[sign-up] {signup_body}");
    assert!(signup_body["access_token"].as_str().is_some());
    assert!(signup_body["refresh_token"].as_str().is_some());

    // 3. /check again — now expect true
    let check2_response = client
        .post(format!("{address}/auth/kakao/check"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let check2_body: Value = check2_response.json().await.unwrap();
    println!("[check #2] {check2_body}");
    assert_eq!(check2_body["registered"], true);

    // 4. /login — already registered, returns fresh tokens
    let login_response = client
        .post(format!("{address}/auth/kakao/login"))
        .bearer_auth(&token)
        .json(&json!({ "fcm_token": "updated-fcm-456" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_response.status(), 200, "login returned non-200");
    let login_body: Value = login_response.json().await.unwrap();
    println!("[login] {login_body}");

    // 5. /reissue — new login's refresh should work, sign-up's should be revoked
    let signup_refresh = signup_body["refresh_token"].as_str().unwrap();
    let revoked = client
        .post(format!("{address}/auth/reissue"))
        .bearer_auth(signup_refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(revoked.status(), 401, "old refresh should be revoked");
    println!(
        "[reissue with old refresh] {} (correctly rejected)",
        revoked.status()
    );

    let login_refresh = login_body["refresh_token"].as_str().unwrap();
    let reissue_response = client
        .post(format!("{address}/auth/reissue"))
        .bearer_auth(login_refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(reissue_response.status(), 200, "reissue returned non-200");
    let reissue_body: Value = reissue_response.json().await.unwrap();
    println!("[reissue] {reissue_body}");

    // 6. /logout
    let final_refresh = reissue_body["refresh_token"].as_str().unwrap();
    let logout_response = client
        .post(format!("{address}/auth/logout"))
        .bearer_auth(final_refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(logout_response.status(), 204, "logout returned non-204");
    println!("[logout] {}", logout_response.status());

    // 7. /reissue with logged-out refresh should fail
    let post_logout = client
        .post(format!("{address}/auth/reissue"))
        .bearer_auth(final_refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(post_logout.status(), 401);
    println!(
        "[reissue after logout] {} (correctly rejected)",
        post_logout.status()
    );

    println!("\n✓ all real-Kakao auth flow assertions passed");
}
