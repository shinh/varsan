extern crate colored;
use colored::*;

pub fn info(msg: String) {
    println!("{}", msg.white());
}

#[macro_export]
macro_rules! log_info {
    ($msg:expr) => (log::info(format!($msg)));
    ($fmt:expr, $($arg:tt)*) => {
        log::info(format!($fmt, $($arg)*));
    }
}
