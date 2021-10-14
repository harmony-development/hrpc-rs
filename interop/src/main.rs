use hrpc::{
    return_error,
    server::{
        error::{json_err_bytes, CustomError, ServerError as HrpcServerError},
        socket::Socket,
        HrpcLayer, HttpResponse, Server, StatusCode,
    },
    IntoResponse, Request, Response,
};
use tower::limit::RateLimitLayer;

use std::{
    fmt::{self, Display, Formatter},
    time::{Duration, Instant},
};

hrpc::include_proto!("test");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("hyper=error".parse().unwrap()),
        )
        .init();

    let is_server = std::env::args().nth(1).map_or(false, |t| t == "server");
    if is_server {
        server().await
    } else {
        client().await
    }
}

async fn client() {
    let mut client = mu_client::MuClient::new("http://localhost:2289".parse().unwrap()).unwrap();

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
        .serve(([127, 0, 0, 1], 2289))
        .await
        .unwrap();
}

#[derive(Debug, Clone)]
struct MuService;

#[hrpc::exports::async_trait]
impl mu_server::Mu for MuService {
    async fn mu(&mut self, request: Request<Ping>) -> Result<Response<Pong>, HrpcServerError> {
        let msg = request.into_message().await?;
        if msg.mu.is_empty() {
            return Err(ServerError::PingEmpty.into());
        }
        Ok((Pong { mu: msg.mu }).into_response())
    }

    fn mu_mute_on_upgrade(&mut self, response: HttpResponse) -> HttpResponse {
        response
    }

    fn mu_mute_middleware(&self, _endpoint: &'static str) -> HrpcLayer {
        HrpcLayer::new(RateLimitLayer::new(1, Duration::from_secs(5)))
    }

    async fn mu_mute(
        &mut self,
        _request: Request<()>,
        sock: Socket<Ping, Pong>,
    ) -> Result<(), HrpcServerError> {
        let periodic_task = {
            let sock = sock.clone();
            async move {
                let mut int = tokio::time::interval(Duration::from_secs(10));
                loop {
                    int.tick().await;
                    return_error!(
                        sock.send_message(Pong {
                            mu: "been 10 seconds".to_string(),
                        })
                        .await
                    );
                }
            }
        };
        let recv_task = async move {
            loop {
                let message = return_error!(sock.receive_message().await);
                return_error!(sock.send_message(Pong { mu: message.mu }).await);
            }
        };
        let (res, res2) = tokio::join!(periodic_task, recv_task);
        res?;
        res2?;
        Ok(())
    }

    async fn mu_mu(
        &mut self,
        request: Request<()>,
        _socket: Socket<Ping, Pong>,
    ) -> Result<(), HrpcServerError> {
        println!("mu_mu: {:?}", request);
        Ok(())
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
    fn as_status_message(&self) -> (StatusCode, prost::bytes::Bytes) {
        let body = json_err_bytes(self.to_string()).into();
        match self {
            ServerError::PingEmpty => (StatusCode::BAD_REQUEST, body),
        }
    }
}
