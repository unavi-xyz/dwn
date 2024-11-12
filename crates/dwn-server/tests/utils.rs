use std::net::SocketAddr;

use dwn::{core::store::RecordStore, stores::NativeDbStore, Actor, Dwn};
use tokio::net::TcpListener;
use xdid::methods::key::{p256::P256KeyPair, DidKeyPair, PublicKey};

pub async fn init_test() -> (Actor, Dwn, impl RecordStore) {
    let remote_store = NativeDbStore::new_in_memory().unwrap();
    let remote_dwn = Dwn::from(remote_store.clone());
    let remote = start_dwn_server(remote_dwn).await;

    let store = NativeDbStore::new_in_memory().unwrap();
    let mut dwn = Dwn::from(store);
    dwn.remote = Some(remote);

    let key = P256KeyPair::generate();
    let did = key.public().to_did();

    let mut actor = Actor::new(did.clone());
    actor.auth_key = Some(key.clone().into());
    actor.sign_key = Some(key.into());

    (actor, dwn, remote_store)
}

pub async fn start_dwn_server(dwn: Dwn) -> String {
    let router = dwn_server::create_router(dwn);

    let port = port_check::free_local_port().unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = addr.to_string();

    tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    format!("http://{}", url)
}
