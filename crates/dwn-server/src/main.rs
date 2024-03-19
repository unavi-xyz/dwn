use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = dwn_server::router();

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
