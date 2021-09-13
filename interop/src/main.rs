use hrpc::{
    return_print,
    server::{
        filters::rate::{rate_limit, Rate},
        json_err_bytes, ServerError as HrpcServerError, Socket, StatusCode,
    },
    warp::{
        self,
        reply::Response,
        {filters::BoxedFilter, Filter},
    },
    Request,
};

use std::{
    fmt::{self, Display, Formatter},
    time::{Duration, Instant},
};

hrpc::include_proto!("test");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt().init();

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
    mu_server::MuServer::new(Server)
        .serve::<ServerError, _>(([127, 0, 0, 1], 2289))
        .await
}

#[derive(Debug)]
struct Server;

#[hrpc::async_trait]
impl mu_server::Mu for Server {
    type Error = ServerError;

    fn middleware(&self, endpoint: &'static str) -> BoxedFilter<()> {
        warp::any()
            .map(move || {
                println!("got {} request", endpoint);
            })
            .untuple_one()
            .boxed()
    }

    fn mu_middleware(&self, _endpoint: &'static str) -> BoxedFilter<()> {
        rate_limit(Rate::new(1, Duration::from_secs(5)), ServerError::TooFast).boxed()
    }

    async fn mu(&self, request: Request<Ping>) -> Result<Pong, HrpcServerError<Self::Error>> {
        let (body, _, _) = request.into_parts();
        let msg = body.into_message().await??;

        if msg.mu.is_empty() {
            return Err(ServerError::PingEmpty.into());
        }
        Ok(Pong { mu: msg.mu })
    }

    fn mu_mute_on_upgrade(&self, response: Response) -> Response {
        response
    }

    type MuMuteValidationType = ();

    async fn mu_mute_validation(
        &self,
        _request: Request<Option<Ping>>,
    ) -> Result<Self::MuMuteValidationType, HrpcServerError<Self::Error>> {
        Ok(())
    }

    async fn mu_mute(
        &self,
        _validation_value: Self::MuMuteValidationType,
        sock: Socket<Ping, Pong>,
    ) {
        let periodic_task = {
            let sock = sock.clone();
            async move {
                let mut int = tokio::time::interval(Duration::from_secs(10));
                loop {
                    int.tick().await;
                    return_print!(
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
                return_print!(sock.receive_message().await, |req| {
                    return_print!(sock.send_message(Pong { mu: req.mu }).await);
                });
            }
        };
        tokio::join!(periodic_task, recv_task);
    }

    type MuMuValidationType = Ping;

    async fn mu_mu_validation(
        &self,
        request: Request<Option<Ping>>,
    ) -> Result<Self::MuMuValidationType, HrpcServerError<Self::Error>> {
        Ok(request
            .into_parts()
            .0
            .into_optional_message()
            .await??
            .unwrap_or_default())
    }

    async fn mu_mu(&self, validation_value: Self::MuMuValidationType, _socket: Socket<Ping, Pong>) {
        println!("mu_mu: {:?}", validation_value);
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
