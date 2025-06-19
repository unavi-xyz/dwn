use thiserror::Error;
use xdid_core::{did::Did, document::Document, Method, ResolutionError};

/// Resolves DIDs using a set of provided methods.
pub struct DidResolver {
    pub methods: Vec<Box<dyn Method>>,
}

impl DidResolver {
    /// Creates a new resolver with all enabled methods.
    pub fn new() -> Result<Self, MethodError> {
        let methods: Vec<Box<dyn Method>> = vec![
            #[cfg(feature = "did-key")]
            Box::new(xdid_method_key::MethodDidKey),
            #[cfg(feature = "did-web")]
            Box::new(xdid_method_web::MethodDidWeb::new()?),
        ];

        Ok(Self { methods })
    }
}

#[derive(Error, Debug)]
pub enum MethodError {
    #[cfg(feature = "did-web")]
    #[error("failed to construct did:web resolver: {0}")]
    DidWeb(#[from] xdid_method_web::reqwest::Error),
}

impl DidResolver {
    pub async fn resolve(&self, did: &Did) -> Result<Document, ResolutionError> {
        for method in self.methods.iter() {
            if method.method_name() == did.method_name.0 {
                return method.resolve(did.clone()).await;
            }
        }

        Err(ResolutionError::UnsupportedMethod)
    }
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, str::FromStr, sync::Arc};

    use hyper::{server::conn::http1::Builder, service::service_fn, Response};
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpListener;
    use xdid_method_key::{p256::P256KeyPair, DidKeyPair, PublicKey};

    use super::*;

    #[tokio::test]
    async fn test_resolve_did_key() {
        let did = P256KeyPair::generate().public().to_did();
        let resolver = DidResolver::new().unwrap();
        let document = resolver.resolve(&did).await.unwrap();
        assert_eq!(document.id, did);
    }

    #[tokio::test]
    async fn test_resolve_did_web() {
        let did = serve_did_web().await;
        let resolver = DidResolver::new().unwrap();
        let document = resolver.resolve(&did).await.unwrap();
        assert_eq!(document.id, did);
    }

    async fn serve_did_web() -> Did {
        let port = port_check::free_local_port().unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await.unwrap();

        let did = Did::from_str(&format!("did:web:localhost%3A{}", port)).unwrap();

        let doc = Document {
            id: did.clone(),
            also_known_as: None,
            assertion_method: None,
            authentication: None,
            capability_delegation: None,
            capability_invocation: None,
            controller: None,
            key_agreement: None,
            service: None,
            verification_method: None,
        };

        let data = Arc::new(serde_json::to_string(&doc).unwrap());

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
        println!("Serving {} at {}", did, url);

        did
    }
}
