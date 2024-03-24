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
    let store = SurrealStore(Arc::new(db));
    let dwn = Arc::new(DWN::new(store));

    let router = dwn_server::router(dwn);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
