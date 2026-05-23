use dearday::run;
use sqlx::PgPool;
use tokio::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    #[allow(dead_code)]
    pub db: PgPool,
}

pub async fn spawn_app(db: PgPool) -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(run(listener, db.clone()));

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        client: reqwest::Client::new(),
        db,
    }
}
