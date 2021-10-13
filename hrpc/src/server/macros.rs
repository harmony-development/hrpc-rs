/// Returns an error.
#[macro_export]
macro_rules! return_error {
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

/// Returns an error as response. Mostly useful for implementing hRPC handlers.
#[macro_export]
macro_rules! return_err_as_resp {
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

/// Combines a list of services that implement `MakeRouter`.
#[macro_export]
macro_rules! combine_services {
    ($fsvc:ident, $($svc:ident),+) => {
        {
            use $crate::server::MakeRouter;

            $fsvc
            $(
                .combine_with($svc)
            )+
        }
    };
}
