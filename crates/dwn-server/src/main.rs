use std::sync::Arc;

use dwn::{store::SurrealDB, DWN};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = SurrealDB::new().await.unwrap();
    let dwn = Arc::new(DWN::new(db));

    let router = dwn_server::router(dwn);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
