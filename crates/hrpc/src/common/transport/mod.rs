/// Common code to work with the HTTP transport.
#[cfg(feature = "_common_http")]
pub mod http;

/// Common code to work with tokio_tungstenite WebSockets.
#[cfg(feature = "websocket_tokio_tungstenite")]
pub mod tokio_tungstenite;

/// Common code to work with ws_stream_wasm WebSockets.
#[cfg(feature = "websocket_wasm")]
pub mod ws_wasm;
