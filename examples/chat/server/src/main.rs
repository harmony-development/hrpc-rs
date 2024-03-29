use std::time::Duration;

use chat_common::{
    chat::{chat_server::*, *},
    BoxError,
};
use hrpc::{
    bail,
    common::layer::trace::TraceLayer,
    exports::http::StatusCode,
    server::{
        layer::ratelimit::RateLimitLayer,
        prelude::*,
        transport::http::{layer::errid_to_status::ErrorIdentifierToStatusLayer, Hyper},
    },
};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

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
    fn send_message_middleware(&self) -> Option<HrpcLayer> {
        // Limit send_message calls to 5 per 2 seconds
        Some(RateLimitLayer::new(5, Duration::from_secs(2)).into_hrpc_layer())
    }

    #[handler]
    async fn send_message(&self, request: Request<Message>) -> ServerResult<Response<Empty>> {
        // Extract the message from the request
        let message = request.into_message().await?;

        if message.content.is_empty() {
            bail!(("empty-message", "empty messages aren't allowed"));
        }

        // Log message content
        tracing::info!("got message: {}", message.content);

        // Try to broadcast the message, if it fails return an error
        self.message_broadcast
            .send(message)
            .map_err(|_| HrpcError::new_internal_server_error("couldn't broadcast message"))?;

        Ok((Empty {}).into_response())
    }

    fn stream_messages_middleware(&self) -> Option<HrpcLayer> {
        // Limit stream_messages calls to 1 per 5 seconds
        Some(RateLimitLayer::new(1, Duration::from_secs(5)).into_hrpc_layer())
    }

    #[handler]
    async fn stream_messages(
        &self,
        _request: Request<()>,
        mut socket: Socket<Message, Empty>,
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
    // Set up logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Create our chat service
    let service = ChatServer::new(ChatService::new());
    // Layer our service with error identifier to HTTP status layer, which helps us convert our error identifers
    // to HTTP statuses.
    let service = service.layer(ErrorIdentifierToStatusLayer::new(|id| {
        id.eq("empty-message").then(|| StatusCode::BAD_REQUEST)
    }));
    // Layer our service with a tracing layer.
    let service = service.layer(TraceLayer::default_debug());

    // Create our transport that we will use to serve our service
    let transport = Hyper::new("127.0.0.1:2289")?;

    // Layer our transport with a CORS header
    //
    // Since this is specific to HTTP, we use the transport's layer method!
    let transport = transport.layer(CorsLayer::permissive());

    // Serve our service with our transport
    transport.serve(service).await?;

    Ok(())
}
