# hrpc

This crate contains generic server and client implementations, and transports
for hRPC. It mainly utilizes `tower`. It uses `matchit` for routing on server.

Currently implemented transports are:
+ HTTP
    - `hyper` client and server for use on native platforms
    - WASM client for use on web
+ Mock client and server (see examples for usage)