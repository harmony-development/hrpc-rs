include!(concat!(env!("OUT_DIR"), "/test.rs"));

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
}
