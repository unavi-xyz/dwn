use std::net::SocketAddr;

use dwn::{Actor, Dwn, core::store::RecordStore, stores::NativeDbStore};
use tokio::net::TcpListener;
use xdid::methods::{
    key::{DidKeyPair, PublicKey, p256::P256KeyPair},
    web::reqwest::Url,
};

pub async fn init_remote_test() -> (Actor, Dwn, impl RecordStore) {
    let remote_store = NativeDbStore::new_in_memory().unwrap();
    let remote_dwn = Dwn::from(remote_store.clone());
    let remote = start_dwn_server(remote_dwn).await;

    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let key = P256KeyPair::generate();
    let did = key.public().to_did();

    let mut actor = Actor::new(did, dwn.clone());
    actor.auth_key = Some(key.clone().into());
    actor.sign_key = Some(key.into());
    actor.remote = Some(remote);

    (actor, dwn, remote_store)
}

pub async fn start_dwn_server(dwn: Dwn) -> Url {
    let router = dwn_server::create_router(dwn);

    let port = port_check::free_local_port().unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Url::parse(&format!("http://{}", addr)).unwrap()
}
