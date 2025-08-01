use std::{net::SocketAddr, sync::Arc};

use dwn::{Actor, Dwn, stores::NativeDbStore};
use hyper::{Response, server::conn::http1::Builder, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::info;
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

pub fn init_dwn() -> (Actor, Dwn) {
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let key = P256KeyPair::generate();
    let did = key.public().to_did();

    let mut actor = Actor::new(did.clone());
    actor.auth_key = Some(key.clone().into());
    actor.sign_key = Some(key.into());

    (actor, dwn)
}

/// Hosts data over HTTP at a random port.
/// Returns a URI to the data.
pub async fn serve_string(data: String) -> String {
    let data = Arc::new(data);

    let port = port_check::free_local_port().unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await.unwrap();

    let handler = move |_| {
        let data = data.clone();
        async move { Ok::<_, hyper::Error>(Response::new(data.to_string())) }
    };

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            if let Err(e) = Builder::new()
                .serve_connection(io, service_fn(&handler))
                .await
            {
                panic!("Error serving connection: {:?}", e);
            }
        }
    });

    let url = format!("http://{}", addr);
    info!("Serving string to {}", url);

    url
}
