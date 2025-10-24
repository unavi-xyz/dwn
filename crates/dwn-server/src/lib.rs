//! DWN HTTP server, using [axum](https://github.com/tokio-rs/axum).
//!
//! The DWN spec does not define a standard HTTP API, so this server is simply built
//! to be compatible with the [dwn](https://github.com/unavi-xyz/dwn/tree/main/crates/dwn)
//! crate.

use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
    sync::LazyLock,
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
use tracing::{debug, error, info};
use xdid::core::did::Did;

pub use dwn::core::reply::Reply;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "dwn-server").expect("project dirs");
    std::fs::create_dir_all(dirs.data_dir()).expect("data dir");
    dirs
});

const DB_FILE: &str = "data.db";

pub struct DwnServerOptions {
    pub port: u16,
    pub in_memory: bool,
}

pub async fn run_server(opts: DwnServerOptions) -> anyhow::Result<()> {
    let store = if opts.in_memory {
        dwn_native_db::NativeDbStore::new_in_memory()?
    } else {
        let mut path = DIRS.data_dir().to_path_buf();
        path.push(DB_FILE);
        dwn_native_db::NativeDbStore::new(path)?
    };
    let dwn = Dwn::from(store);

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, opts.port));
    let listener = TcpListener::bind(addr).await?;
    let router = create_router(dwn);

    info!("DWN server running on port {}", opts.port);

    axum::serve(listener, router).await?;

    Ok(())
}

pub fn create_router(dwn: Dwn) -> Router {
    Router::new()
        .route("/{target}", put(handle_put))
        .with_state(dwn)
}

#[debug_handler]
async fn handle_put(
    Path(mut target): Path<String>,
    State(dwn): State<Dwn>,
    Json(msg): Json<Message>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if target.starts_with("did:web:") {
        // Axum automatically decodes percent-encoded paths.
        // However, for did:web if a port is included the colon must remain percent-encoded.
        let (_, rest) = target
            .split_once("did:web:")
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut parts = rest.split(':');

        if let Some(first) = parts.next()
            && let Some(second) = parts.next()
            && second.len() <= 5
            && second.chars().all(|c| c.is_numeric())
        {
            // Assume second is a port.
            let parts_vec = parts.collect::<Vec<_>>();
            let parts_str = if parts_vec.is_empty() {
                String::new()
            } else {
                format!(":{}", parts_vec.join(":"))
            };
            target = format!("did:web:{first}%3A{second}{parts_str}");
        }
    }

    // debug!("-> PUT {target}");

    let target = Did::from_str(&target).map_err(|e| {
        debug!("Failed to parse DID: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    let reply = dwn.process_message(&target, msg).await?;

    let res = serde_json::to_value(reply).map_err(|e| {
        error!("Error serializing response: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // debug!("<- {res}");

    Ok(Json(res))
}
