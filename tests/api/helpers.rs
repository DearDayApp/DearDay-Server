use dearday::{AppState, Config, run};
use sqlx::PgPool;
use tokio::net::TcpListener;
use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    #[allow(dead_code)]
    pub db: PgPool,
    pub kakao_mock: MockServer,
}

pub async fn spawn_app(db: PgPool) -> TestApp {
    let kakao_mock = MockServer::start().await;

    let config = Config {
        bind_addr: "127.0.0.1:0".into(),
        database_url: "unused-in-tests".into(),
        jwt_access_secret: "test-access-secret-must-be-long-enough".into(),
        jwt_refresh_secret: "test-refresh-secret-must-be-long-enough".into(),
        kakao_api_url: kakao_mock.uri(),
    };

    let state = AppState::new(&config, db.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(run(listener, state));

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        client: reqwest::Client::new(),
        db,
        kakao_mock,
    }
}
