use hrpc::{
    return_print,
    server::{
        filters::rate::{rate_limit, Rate},
        json_err_bytes, Socket, StatusCode,
    },
    warp::reply::Response,
    Request,
};
use warp::{filters::BoxedFilter, Filter};

use std::{
    fmt::{self, Display, Formatter},
    time::{Duration, Instant},
};

hrpc::include_proto!("test");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt().pretty().init();

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

    let mut socket = client.mu_mute(()).await.unwrap();

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

    socket.close().await.unwrap();
}

async fn server() {
    mu_server::MuServer::new(Server)
        .serve::<ServerError, _>(([127, 0, 0, 1], 2289))
        .await
}

#[derive(Debug)]
struct Server;

#[hrpc::async_trait]
impl mu_server::Mu for Server {
    type Error = ServerError;

    fn mu_pre(&self) -> BoxedFilter<(Result<(), ServerError>,)> {
        warp::any()
            .and(rate_limit(
                Rate::new(1, Duration::from_secs(5)),
                ServerError::TooFast,
            ))
            .boxed()
    }

    async fn mu(&self, request: Request<Ping>) -> Result<Pong, Self::Error> {
        if request.get_message().mu.is_empty() {
            return Err(ServerError::PingEmpty);
        }
        Ok(Pong {
            mu: request.into_parts().0.mu,
        })
    }

    fn mu_mute_on_upgrade(&self, response: Response) -> Response {
        response
    }

    async fn mu_mute(&self, _validation_request: Request<()>, mut socket: Socket<Ping, Pong>) {
        let mut creation_timestamp = Instant::now();
        while let Ok(request) = socket.receive_message().await {
            if let Some(req) = request {
                return_print!(socket.send_message(Pong { mu: req.mu }).await).unwrap();
            }

            if creation_timestamp.elapsed().as_secs() >= 10 {
                creation_timestamp = Instant::now();
                return_print!(
                    socket
                        .send_message(Pong {
                            mu: "been 10 seconds".to_string(),
                        })
                        .await
                )
                .unwrap();
            }
        }
    }
}

#[derive(Debug)]
enum ServerError {
    PingEmpty,
    TooFast(Duration),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ServerError::PingEmpty => write!(f, "sent empty ping"),
            ServerError::TooFast(try_again) => {
                write!(f, "too fast, try again in {}", try_again.as_secs_f64())
            }
        }
    }
}

impl hrpc::server::CustomError for ServerError {
    fn code(&self) -> StatusCode {
        match self {
            ServerError::PingEmpty => StatusCode::BAD_REQUEST,
            ServerError::TooFast(_) => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn message(&self) -> Vec<u8> {
        json_err_bytes(&self.to_string())
    }
}
