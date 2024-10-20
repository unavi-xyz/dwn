use std::{net::SocketAddr, sync::Arc};

use hyper::{server::conn::http1::Builder, service::service_fn, Response};
use hyper_util::rt::TokioIo;
use port_check::free_local_port;
use tokio::net::TcpListener;
use tracing::info;

/// Hosts data over HTTP at a random port.
/// Returns a URI to the data.
pub async fn serve_string(data: String) -> String {
    let data = Arc::new(data);

    let port = free_local_port().unwrap();
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