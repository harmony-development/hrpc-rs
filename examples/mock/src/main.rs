use hrpc::{
    client::transport::mock::Mock as MockClient,
    common::transport::mock::{new_mock_channels, MockReceiver, MockSender},
    server::transport::{mock::Mock as MockServer, Transport},
};
use mock::{
    hello::{greeter_client::GreeterClient, greeter_server::GreeterServer, WelcomeUserRequest},
    server::GreeterService,
    BoxError,
};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // First create mock channels
    let (tx, rx) = new_mock_channels();

    // Then we spawn our server
    spawn_server(rx);

    // Afterwards we make a client
    let mut client = make_client(tx);

    // Now we can call some methods!
    let response = client
        .welcome_user(WelcomeUserRequest {
            user_name: "mock".to_string(),
        })
        .await?;
    let welcome_message = response.into_message().await?.welcome_message;

    println!("mock response: {}", welcome_message);

    Ok(())
}

fn spawn_server(rx: MockReceiver) {
    let transport = MockServer::new(rx);
    let server = GreeterServer::new(GreeterService);
    let fut = transport.serve(server);

    tokio::spawn(fut);
}

fn make_client(tx: MockSender) -> GreeterClient<MockClient> {
    let transport = MockClient::new(tx);
    GreeterClient::new_transport(transport)
}
