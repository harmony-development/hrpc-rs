use hello_world::{
    hello::{greeter_client::GreeterClient, WelcomeUserRequest},
    BoxError,
};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Create a new greeter client
    let mut client = GreeterClient::new("http://localhost:2289")?;

    // Call the welcome user RPC
    let response = client
        .welcome_user(WelcomeUserRequest {
            user_name: "joe".to_string(),
        })
        .await?
        .into_message()
        .await?;

    // Print the welcome message
    println!("{}", response.welcome_message);

    Ok(())
}
