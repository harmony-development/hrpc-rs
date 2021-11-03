use hrpc::{
    bail, bail_result,
    exports::{http::StatusCode, tracing::Level},
    server::prelude::*,
};
use tower::limit::RateLimitLayer;
use tracing_subscriber::{filter::Targets, prelude::*};

use std::{
    fmt::{self, Display, Formatter},
    time::{Duration, Instant},
};

hrpc::include_proto!("test");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let layer = tracing_subscriber::fmt::layer().with_filter(
        Targets::default()
            .with_target("hyper", Level::ERROR)
            .with_default(Level::DEBUG),
    );

    tracing_subscriber::registry().with(layer).init();

    let is_server = std::env::args().nth(1).map_or(false, |t| t == "server");
    if is_server {
        server().await
    } else {
        client().await
    }
}

async fn client() {
    let mut client = mu_client::MuClient::new("http://localhost:2289").unwrap();

    let resp = client
        .mu(Ping {
            mu: "123".to_string(),
        })
        .await;
    println!("{:#?}", resp);

    let resp = client.mu(Ping { mu: "".to_string() }).await;
    println!("{:#?}", resp);

    let socket = client.mu_mute(()).await.unwrap();
    let _ = client.mu_mu(Ping::default()).await.unwrap();

    tokio::spawn({
        let socket = socket.clone();
        async move {
            loop {
                let ins = Instant::now();
                match socket.receive_message().await {
                    Ok(msg) => {
                        println!("got in {}", ins.elapsed().as_secs_f64());
                        println!("{:#?}", msg);
                    }
                    Err(_) => break,
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
        let ins = Instant::now();
        match client.mu(Ping { mu: i.to_string() }).await {
            Err(err) => {
                eprintln!("failed to send message: {}", err);
            }
            Ok(pong) => {
                println!("got {:#?}", pong);
            }
        }
        println!("sent in {}", ins.elapsed().as_secs_f64());

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    socket.close().await;
}

async fn server() {
    mu_server::MuServer::new(MuService)
        .serve("127.0.0.1:2289")
        .await
        .unwrap();
}

#[derive(Debug, Clone)]
struct MuService;

impl mu_server::Mu for MuService {
    #[handler]
    async fn mu_mu(&self, req: Request<()>, _: Socket<Ping, Pong>) -> ServerResult<()> {
        println!("mu_mu: {:?}", req);
        Ok(())
    }

    fn mu_middleware(&self, _endpoint: &'static str) -> Option<HrpcLayer> {
        Some(HrpcLayer::new(RateLimitLayer::new(
            5,
            Duration::from_secs(10),
        )))
    }

    #[handler]
    async fn mu(&self, request: Request<Ping>) -> ServerResult<Response<Pong>> {
        let msg = request.into_message().await?;
        if msg.mu.is_empty() {
            bail!(ServerError::PingEmpty);
        }
        Ok((Pong { mu: msg.mu }).into_response())
    }

    #[handler]
    async fn mu_mute(&self, _: Request<()>, sock: Socket<Ping, Pong>) -> ServerResult<()> {
        let periodic_task = sock.spawn_task(|sock| async move {
            let mut int = tokio::time::interval(Duration::from_secs(10));
            int.tick().await;
            loop {
                int.tick().await;
                bail_result!(
                    sock.send_message(Pong {
                        mu: "been 10 seconds".to_string(),
                    })
                    .await
                );
            }
        });

        let recv_task =
            sock.spawn_process_task(|_sock, message| async { Ok(Pong { mu: message.mu }) });

        tokio::select! {
            res = periodic_task => res.unwrap(),
            res = recv_task => res.unwrap(),
        }
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

impl std::error::Error for ServerError {}

impl CustomError for ServerError {
    fn error_message(&self) -> std::borrow::Cow<'_, str> {
        self.to_string().into()
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::PingEmpty => StatusCode::BAD_REQUEST,
        }
    }
}
