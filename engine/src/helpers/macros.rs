#[macro_export]
macro_rules! check_result {
    ($res:expr, $msg:expr) => {
        if let Err(e) = $res {
            return Err(Error::msg(format!("{}: {}", $msg, e)));
        }
    };

    ($res:expr) => {
        if let Err(e) = $res {
            return Err(Error::msg(format!("I/O error: {}", e)));
        }
    };
}
