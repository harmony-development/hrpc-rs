# mock

This example showcases the mock transport. The mock transport is intended
for tests, where networking might not be available. It uses a simple
`tokio` MPSC channel for communication between client and server.