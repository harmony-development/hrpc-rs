use chat::{
    chat::{chat_server::*, *},
    BoxError,
};
use hrpc::server::prelude::*;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct ChatService {
    message_broadcast: broadcast::Sender<Message>,
}

impl ChatService {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            message_broadcast: tx,
        }
    }
}

impl Chat for ChatService {
    #[handler]
    async fn send_message(&mut self, request: Request<Message>) -> ServerResult<Response<Empty>> {
        // Extract the message from the request
        let message = request.into_message().await?;

        // Try to broadcast the message, fail otherwise and return a custom error
        self.message_broadcast.send(message).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "couldn't broadcast message",
            )
        })?;

        Ok((Empty {}).into_response())
    }

    #[handler]
    async fn stream_messages(
        &mut self,
        _request: Request<()>,
        socket: Socket<Empty, Message>,
    ) -> ServerResult<()> {
        // Subscribe to the message broadcaster
        let mut message_receiver = self.message_broadcast.subscribe();

        // Spawn a task to process the received messages and send them to client
        let task = socket.spawn_task(move |socket| async move {
            while let Ok(message) = message_receiver.recv().await {
                socket.send_message(message).await?;
            }

            Ok(())
        });

        task.await.unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Create our chat service
    let service = ChatService::new();

    // Serve our service
    ChatServer::new(service).serve("127.0.0.1:2289").await?;

    Ok(())
}
