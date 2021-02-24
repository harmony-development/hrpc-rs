use hrpc::server::StatusCode;
use simplelog::Config;
use std::{
    fmt::{self, Display, Formatter},
    time::Duration,
};

hrpc::include_proto!("test");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
        simplelog::LevelFilter::Info,
        Config::default(),
        simplelog::TerminalMode::Mixed,
    )])
    .unwrap();

    let is_server = std::env::args().nth(1).map_or(false, |t| t == "server");
    if is_server {
        server().await
    } else {
        client().await
    }
}

async fn client() {
    let mut client = mu_client::MuClient::new(
        hrpc::reqwest::Client::new(),
        "http://localhost:2289".parse().unwrap(),
    )
    .unwrap();

    let resp = client
        .mu(Ping {
            mu: "123".to_string(),
        })
        .await;
    println!("{:#?}", resp);

    let resp = client.mu(Ping { mu: "".to_string() }).await;
    println!("{:#?}", resp);

    let mut socket = client.mu_mute().await.unwrap();

    for i in 0..100 {
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

async fn server() {
    mu_server::MuServer::new(Server)
        .serve(([127, 0, 0, 1], 2289))
        .await
}

#[derive(Debug)]
struct Server;

#[hrpc::async_trait]
impl mu_server::Mu for Server {
    const PING_PERIOD: u64 = 15;

    type Error = ServerError;

    async fn mu(&self, request: Ping) -> Result<Pong, Self::Error> {
        if request.mu.is_empty() {
            return Err(ServerError::PingEmpty);
        }
        Ok(Pong { mu: request.mu })
    }

    async fn mu_mute(&self, request: Ping) -> Result<Pong, Self::Error> {
        self.mu(request).await
    }
}

#[derive(Debug)]
enum ServerError {
    PingEmpty,
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ServerError::PingEmpty => write!(f, "sent empty ping"),
        }
    }
}

impl hrpc::server::CustomError for ServerError {
    fn code(&self) -> StatusCode {
        match self {
            ServerError::PingEmpty => StatusCode::BAD_REQUEST,
        }
    }

    fn message(&self) -> &str {
        match self {
            ServerError::PingEmpty => "sent empty ping",
        }
    }
}
