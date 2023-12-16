#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dwn::server().await;
}
