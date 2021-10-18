/// Bails with an error.
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into());
    };
}

/// Takes a `Result`, returns the error if it's `Err`, otherwise returns the
/// `Ok` value.
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
#[macro_export]
macro_rules! combine_services {
    ($fsvc:ident, $($svc:ident),+) => {
        {
            use $crate::server::Server;

            $fsvc
            $(
                .combine_with($svc)
            )+
        }
    };
}
