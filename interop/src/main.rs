use hrpc::server::{json_err_bytes, StatusCode};
use simplelog::Config;
use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
    time::{Duration, Instant},
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

    let socket = Arc::new(client.mu_mute(()).await.unwrap());

    tokio::spawn({
        let socket = socket.clone();
        async move {
            loop {
                let ins = Instant::now();
                match socket.get_message().await {
                    Some(Ok(msg)) => {
                        println!("got in {}", ins.elapsed().as_secs_f64());
                        println!("{:#?}", msg);
                    }
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    });

    for i in 0..100 {
        let ins = Instant::now();
        if let Err(err) = socket.send_message(Ping { mu: i.to_string() }).await {
            eprintln!("failed to send message: {}", err);
        }
        println!("sent in {}", ins.elapsed().as_secs_f64());

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    socket.close().await.unwrap();
}

async fn server() {
    mu_server::MuServer::new(Server {
        creation_timestamp: Instant::now(),
    })
    .serve(([127, 0, 0, 1], 2289))
    .await
}

#[derive(Debug)]
struct Server {
    creation_timestamp: Instant,
}

#[hrpc::async_trait]
impl mu_server::Mu for Server {
    type Error = ServerError;

    async fn mu(&self, request: Ping) -> Result<Pong, Self::Error> {
        if request.mu.is_empty() {
            return Err(ServerError::PingEmpty);
        }
        Ok(Pong { mu: request.mu })
    }

    async fn mu_mute(&self, request: Option<Ping>) -> Result<Option<Pong>, Self::Error> {
        if let Some(req) = request {
            return self.mu(req).await.map(Some);
        }

        Ok(None)
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

    fn message(&self) -> Vec<u8> {
        match self {
            ServerError::PingEmpty => json_err_bytes("sent empty ping"),
        }
    }
}
