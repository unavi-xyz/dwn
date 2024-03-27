use std::sync::Arc;

use dwn::{store::SurrealStore, DWN};
use surrealdb::{engine::local::SpeeDb, Surreal};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Make database directory if it doesn't exist.
    std::fs::create_dir_all("/database").unwrap();

    let db = Surreal::new::<SpeeDb>("/database").await.unwrap();
    let dwn = DWN::from(SurrealStore::from(db));

    let router = dwn_server::router(Arc::new(dwn));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
