//! DWN HTTP server, using [axum](https://github.com/tokio-rs/axum).
//!
//! The DWN spec does not define a standard HTTP API, so this server is simply built
//! to be compatible with the [dwn](https://github.com/unavi-xyz/dwn/tree/main/crates/dwn)
//! crate.

use std::{
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::put,
};
use axum_macros::debug_handler;
use directories::ProjectDirs;
use dwn::{Dwn, core::message::Message};
use tokio::net::TcpListener;
use tracing::{debug, info};
use xdid::core::did::Did;

pub use dwn::core::reply::Reply;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "dwn-server").expect("project dirs");
    std::fs::create_dir_all(dirs.data_dir()).expect("data dir");
    dirs
});

const DB_FILE: &str = "data.db";

pub async fn run_server(addr: SocketAddr) -> anyhow::Result<()> {
    let path = {
        let mut dir = DIRS.data_dir().to_path_buf();
        dir.push(DB_FILE);
        dir
    };
    let store = Arc::new(dwn_native_db::NativeDbStore::new(path)?);
    let dwn = Dwn::new(store.clone(), store);

    let listener = TcpListener::bind(addr).await?;
    let router = create_router(dwn);

    info!("DWN server running on {addr}");

    axum::serve(listener, router).await?;

    Ok(())
}

pub fn create_router(dwn: Dwn) -> Router {
    // TODO: Anyone can write messages to the DWN, even if they are not `target`.
    // Is this a problem? We would need some authentication solution, verifying
    // the requester has permission to write messages.
    Router::new()
        .route("/{target}", put(handle_put))
        .with_state(dwn)
}

#[debug_handler]
async fn handle_put(
    Path(target): Path<String>,
    State(mut dwn): State<Dwn>,
    Json(msg): Json<Message>,
) -> Result<Json<Option<Reply>>, StatusCode> {
    let target = Did::from_str(&target).map_err(|e| {
        debug!("Failed to parse DID: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    let reply = dwn.process_message(&target, msg).await?;

    Ok(Json(reply))
}
