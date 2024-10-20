# dwn-server

<!-- cargo-rdme start -->

DWN HTTP server, using [axum](https://github.com/tokio-rs/axum).

The DWN spec does not define a standard HTTP API, so this server is simply built
to be compatible with the [dwn](https://github.com/unavi-xyz/dwn/tree/main/crates/dwn)
crate.

### Design

The server provides a REST API, leaning into the strengths of HTTP.
For example, using HTTP-level status codes instead of the spec-defined
JSON reply objects.

<!-- cargo-rdme end -->
