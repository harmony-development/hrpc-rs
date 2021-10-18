use hello_world::{
    hello::{
        greeter_server::{Greeter, GreeterServer},
        WelcomeUserRequest, WelcomeUserResponse,
    },
    BoxError,
};
use hrpc::{
    exports::async_trait,
    server::{error::ServerResult, Server},
    IntoResponse, Request, Response,
};

#[derive(Clone)]
pub struct GreeterService;

#[async_trait]
impl Greeter for GreeterService {
    async fn welcome_user(
        &mut self,
        request: Request<WelcomeUserRequest>,
    ) -> ServerResult<Response<WelcomeUserResponse>> {
        // Extract the message from the request
        let message = request.into_message().await?;

        // Craft a response message using the `user_name` we got in the request message
        let response_message = WelcomeUserResponse {
            welcome_message: format!("Welcome, {}!", message.user_name),
        };

        Ok(response_message.into_response())
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Create a new greeter server and start serving it
    GreeterServer::new(GreeterService)
        .serve("127.0.0.1:2289")
        .await?;

    Ok(())
}
