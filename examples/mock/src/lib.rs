/// `hello` package protobuf definitions and service.
pub mod hello {
    hrpc::include_proto!("hello");
}

/// A boxed error.
pub type BoxError = Box<dyn std::error::Error>;

/// Server implementation.
pub mod server {
    use super::hello::{greeter_server::*, *};
    use hrpc::server::prelude::*;

    pub struct GreeterService;

    impl Greeter for GreeterService {
        #[handler]
        async fn welcome_user(
            &self,
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
}
