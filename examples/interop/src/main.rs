use hrpc::{bail, bail_result, exports::tracing::Level, server::prelude::*, BoxError};
use tower::limit::RateLimitLayer;
use tracing_subscriber::{filter::Targets, prelude::*};

use std::time::{Duration, Instant};

hrpc::include_proto!("test");

#[tokio::main]
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
        client().await.unwrap()
    }
}

async fn client() -> Result<(), BoxError> {
    let mut client = mu_client::MuClient::new("http://localhost:2289").unwrap();

    let resp = client
        .mu(Ping {
            mu: "123".to_string(),
        })
        .await?
        .into_message()
        .await?;
    println!("{:#?}", resp);

    let resp = client.mu(Ping { mu: "".to_string() }).await;
    println!("{:#?}", resp);

    let (mut write, mut read) = client.mu_mute(()).await.unwrap().split();
    let _ = client.mu_mu(Ping::default()).await.unwrap();

    tokio::spawn(async move {
        while let Ok(msg) = read.receive_message().await {
            println!("got: {:#?}", msg);
        }
    });

    for i in 0..100 {
        let ins = Instant::now();
        if let Err(err) = write.send_message(Ping { mu: i.to_string() }).await {
            eprintln!("failed to send message: {}", err);
        }
        println!("sent in {}", ins.elapsed().as_secs_f64());
        let ins = Instant::now();
        match client.mu(Ping { mu: i.to_string() }).await {
            Err(err) => {
                eprintln!("failed to send message: {}", err);
            }
            Ok(resp) => {
                println!("got {:#?}", resp.into_message().await?);
            }
        }
        println!("sent in {}", ins.elapsed().as_secs_f64());

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    write.close().await?;

    Ok(())
}

async fn server() {
    mu_server::MuServer::new(MuService)
        .serve("127.0.0.1:2289")
        .await
        .unwrap();
}

struct MuService;

impl mu_server::Mu for MuService {
    #[handler]
    async fn mu_mu(&self, req: Request<()>, _: Socket<Pong, Ping>) -> ServerResult<()> {
        println!("mu_mu: {:?}", req);
        Ok(())
    }

    fn mu_middleware(&self) -> Option<HrpcLayer> {
        Some(HrpcLayer::new(RateLimitLayer::new(
            5,
            Duration::from_secs(10),
        )))
    }

    #[handler]
    async fn mu(&self, request: Request<Ping>) -> ServerResult<Response<Pong>> {
        let msg = request.into_message().await?;
        if msg.mu.is_empty() {
            bail!(("interop.empty-ping", "empty ping"));
        }
        Ok((Pong { mu: msg.mu }).into_response())
    }

    #[handler]
    async fn mu_mute(&self, _: Request<()>, mut sock: Socket<Pong, Ping>) -> ServerResult<()> {
        let mut int = tokio::time::interval(Duration::from_secs(10));
        int.tick().await;

        loop {
            tokio::select! {
                res = sock.receive_message() => {
                    match res {
                        Ok(msg) => println!("{:?}", msg),
                        Err(err) => return Err(err),
                    }
                }
                _ = int.tick() => {
                    bail_result!(
                        sock.send_message(Pong {
                            mu: "been 10 seconds".to_string(),
                        })
                        .await
                    );
                }
            }
        }
    }
}
