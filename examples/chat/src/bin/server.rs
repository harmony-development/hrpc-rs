use chat::{
    chat::{chat_server::*, *},
    BoxError,
};
use hrpc::{
    exports::http::{self, HeaderValue},
    server::prelude::*,
};
use tokio::sync::broadcast;
use tower_http::set_header::SetResponseHeaderLayer;

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
    async fn send_message(&self, request: Request<Message>) -> ServerResult<Response<Empty>> {
        // Extract the message from the request
        let message = request.into_message().await?;

        // Try to broadcast the message, if it fails return an error
        self.message_broadcast
            .send(message)
            .map_err(|_| HrpcError::new_internal_server_error("couldn't broadcast message"))?;

        Ok((Empty {}).into_response())
    }

    #[handler]
    async fn stream_messages(
        &self,
        _request: Request<()>,
        socket: Socket<Empty, Message>,
    ) -> ServerResult<()> {
        // Subscribe to the message broadcaster
        let mut message_receiver = self.message_broadcast.subscribe();

        // Process the received messages and send them to client
        while let Ok(message) = message_receiver.recv().await {
            socket.send_message(message).await?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // CORS layer for adding CORS header
    let cors_layer = HrpcLayer::new(SetResponseHeaderLayer::<_, ()>::if_not_present(
        http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    ));

    // Create our chat service
    let service = ChatServer::new(ChatService::new()).layer(cors_layer);

    // Create our transport that we will use to server our service
    let transport = Hyper::new("127.0.0.1:2289");

    // Serve our service
    transport.serve(service).await?;

    Ok(())
}
