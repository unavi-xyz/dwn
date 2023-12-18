use dwn_server::StartOptions;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dwn_server::start(StartOptions::default()).await;
}
