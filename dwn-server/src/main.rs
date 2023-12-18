#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dwn_server::start().await;
}
