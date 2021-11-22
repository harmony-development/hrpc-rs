/// Bails with an error.
///
/// # Example
/// ```rust,no_run
/// # use hrpc::{bail, proto::Error as HrpcError};
/// # fn main() -> Result<(), HrpcError> {
///     bail!(("some-error", "a very bad error occured!"));
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
///     bail_result!(some_computation());
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! bail_result {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => return Err(err.into()),
        }
    };
    ($res:expr, |$err:ident| $func:expr) => {
        match $res {
            Ok(val) => val,
            Err($err) => {
                $func
                return Err($err.into());
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
            use $crate::server::MakeRoutes;

            $( #[$fattr] )*
            let svc = $fsvc;
            $(
                $( #[$attr] )*
                let svc = MakeRoutes::combine_with(svc, $svc);
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
