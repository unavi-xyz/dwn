//! Simple [DID](https://www.w3.org/TR/did-core/) library.
//!
//! ## Example
//!
//! ```
//! use xdid::{resolver::DidResolver, methods::key::{p256::P256KeyPair, DidKeyPair, PublicKey}};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Generate a new did:key.
//!     let keys = P256KeyPair::generate();
//!     let did = keys.public().to_did();
//!
//!     assert!(did.to_string().starts_with("did:key:zDn"));
//!
//!     // Resolve the DID document.
//!     let resolver = DidResolver::new().unwrap();
//!     let document = resolver.resolve(&did).await.unwrap();
//!
//!     assert_eq!(document.id, did);
//! }
//! ```

pub mod resolver;

pub mod core {
    pub use xdid_core::*;
}

pub mod methods {
    #[cfg(feature = "did-key")]
    pub mod key {
        pub use xdid_method_key::*;
    }

    #[cfg(feature = "did-web")]
    pub mod web {
        pub use xdid_method_web::*;
    }
}
