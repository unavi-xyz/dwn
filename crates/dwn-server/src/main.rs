use clap::Parser;
use dwn_server::DwnServerOptions;
use tracing::{Level, error};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    if let Err(e) = dwn_server::run_server(DwnServerOptions {
        port: args.port,
        in_memory: false,
    })
    .await
    {
        error!("{e:?}");
    }
}
