/// Returns an error.
#[macro_export]
macro_rules! return_error {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => return Err(err),
        }
    };
}
