use hello_world::{
    hello::{greeter_client::GreeterClient, WelcomeUserRequest},
    BoxError,
};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Create a new greeter client
    let mut client = GreeterClient::connect("http://localhost:2289")?;

    // Call the welcome user RPC
    let response = client
        .welcome_user(WelcomeUserRequest {
            user_name: "joe".to_string(),
        })
        .await?;

    // Print the welcome message
    println!("{}", response.welcome_message);

    Ok(())
}
