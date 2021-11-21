# `chat`

This is an example showcasing unary and streaming requests, creating and using
transports and layering as a very simple chatting app.

To run the server:
```console
$ cargo run --package chat_server
```

To run the CLI client:
```console
$ cargo run --package chat_client
```

To run the WASM web client, you will need [`trunk`](https://trunkrs.dev):
```console
$ trunk serve wasm_client/index.html
```