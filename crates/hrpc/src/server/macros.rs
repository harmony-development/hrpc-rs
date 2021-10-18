/// Bails with an error.
///
/// # Example
/// ```rust,no_run
/// # use hrpc::{bail, server::error::ServerError, exports::http::StatusCode};
/// # fn main() -> Result<(), ServerError> {
///     bail!((StatusCode::BAD_REQUEST, "method must be POST"));
/// #   Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into());
    };
}

/// Takes a `Result`, returns the error if it's `Err`, otherwise returns the
/// `Ok` value.
///
/// # Example
/// ```rust,ignore
/// async fn handler(&mut self, request: Request<TRequest>) -> ServerResult<Response<TResponse>> {
///     // ...
///     let some_computation_result = some_computation();
///     bail_result!(some_computation_result);
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! bail_result {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => return Err(err),
        }
    };
    ($res:expr, |$err:ident| $func:expr) => {
        match $res {
            Ok(val) => val,
            Err($err) => {
                $func
                return Err(err);
            },
        }
    };
}

/// Takes a `Result`, returns the error as a HTTP response if it's `Err`,
/// otherwise returns the `Ok` value.
#[macro_export]
macro_rules! bail_result_as_response {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => return Ok($crate::server::error::CustomError::as_error_response(&err)),
        }
    };
    ($res:expr, |$err:ident| $func:expr) => {
        match $res {
            Ok(val) => val,
            Err($err) => {
                $func
                return Ok($crate::server::error::CustomError::as_error_response(&$err));
            },
        }
    };
}

/// Combines a list of services that implement `Server`.
///
/// # Example
/// ```rust,ignore
/// let hello = HelloServer::new(HelloService);
/// let welcome = WelcomeServer::new(WelcomeService);
/// combine_services!(hello, welcome).serve("127.0.0.1:2289").await?;
/// ```
#[macro_export]
macro_rules! combine_services {
    ($fsvc:ident, $($svc:ident),+) => {
        {
            use $crate::server::Server;

            let svc = $fsvc;
            $(
                let svc = Server::combine_with(svc, $svc);
            )+
            svc
        }
    };
}
