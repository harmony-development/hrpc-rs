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

/// Combines a list of services that implement `MakeRoutes`.
///
/// # Example
/// ```rust,ignore
/// let hello = HelloServer::new(HelloService);
/// let welcome = WelcomeServer::new(WelcomeService);
/// combine_services!(hello, welcome).serve("127.0.0.1:2289").await?;
/// ```
#[macro_export]
macro_rules! combine_services {
    (
        $( #[$fattr:meta] )*
        $fsvc:ident,
        $(
            $( #[$attr:meta] )*
            $svc:ident
        ),+
    ) => {
        {
            use $crate::server::Service;

            $( #[$fattr] )*
            let svc = $fsvc;
            $(
                $( #[$attr] )*
                let svc = Service::combine_with(svc, $svc);
            )+
            svc
        }
    };
}

/// Macro to workaround `async fn`s not being allowed in traits. You do not
/// need to use this directly, instead you should use the `handler` macro
/// attribute provided in the server prelude.
#[macro_export]
macro_rules! make_handler {
    (
        $( #[$attr:meta] )*
        $pub:vis
        async
        fn $fname:ident ( $($args:tt)* ) $(-> $Ret:ty)?
        {
            $($body:tt)*
        }
    ) => {
        $( #[$attr] )*
        $pub
        fn $fname ( $($args)* ) -> $crate::exports::futures_util::future::BoxFuture<'_, ($($Ret)?)> {
            Box::pin(async move { $($body)* })
        }
    };
}
