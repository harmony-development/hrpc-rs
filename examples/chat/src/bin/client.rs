use chat::{
    chat::{chat_client::ChatClient, *},
    BoxError,
};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Create a new chat client
    let mut client = ChatClient::new("http://localhost:2289")?;

    // Connect to message socket
    let socket = client.stream_messages(Empty {}).await?;

    // Send a message
    client
        .send_message(Message {
            content: "hello world!".to_string(),
        })
        .await?;

    // Wait messages and post them
    while let Ok(message) = socket.receive_message().await {
        println!("got: {}", message.content);
    }

    Ok(())
}
