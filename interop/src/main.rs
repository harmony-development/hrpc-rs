use std::time::Duration;

hrpc::include_proto!("test");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut client = mu_client::MuClient::new(
        hrpc::reqwest::Client::new(),
        "http://localhost:9999".parse().unwrap(),
    )
    .unwrap();

    let resp = client
        .mu(Ping {
            mu: "123".to_string(),
        })
        .await;

    println!("{:#?}", resp);

    let mut socket = client.mu_mute().await.unwrap();

    for i in 0..5 {
        if let Err(err) = socket.send_message(Ping { mu: i.to_string() }).await {
            eprintln!("failed to send message: {}", err);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;

        if let Some(Ok(msg)) = socket.get_message().await {
            println!("{:#?}", msg);
        }
    }

    socket.close().await.unwrap();
}
