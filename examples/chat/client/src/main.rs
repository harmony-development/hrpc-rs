use chat_common::{
    chat::{chat_client::ChatClient, *},
    BoxError,
};

use hrpc::client::transport::http::Hyper;
use rustyline::{error::ReadlineError, Editor as Rustyline};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Create a new chat client
    let transport = Hyper::new("http://localhost:2289".parse()?)?;
    let mut client = ChatClient::new_transport(transport);

    // Connect to message socket
    let mut socket = client.stream_messages(Empty {}).await?;

    // Send a message
    client
        .send_message(Message {
            content: "hello world!".to_string(),
        })
        .await?;

    // Wait for messages and post them, in a seperate task
    tokio::spawn(async move {
        while let Ok(message) = socket.receive_message().await {
            println!("got: {}", message.content);
        }
    });

    // Create our rustyline instance which we will use to read messages
    // from stdin
    let mut rustyline = Rustyline::<()>::new();
    loop {
        let readline = rustyline.readline("(write your message)> ");
        match readline {
            Ok(line) => {
                client.send_message(Message { content: line }).await?;
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("rustyline error: {}", err);
                break;
            }
        }
    }

    Ok(())
}
