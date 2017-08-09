extern crate libc;

use std;

pub fn errno() -> i32 {
    unsafe {
        return *libc::__errno_location();
    }
}

pub fn abort_libc(msg: &str) {
    let strerror: String;
    unsafe {
        strerror = std::ffi::CString::from_raw(
            libc::strerror(errno())).into_string().unwrap();
    }
    panic!("{}: {} (errno={})", msg, strerror, errno());
}

pub fn check_libc<'a>(retval: i32, msg: &'a str) {
    if retval < 0 {
        abort_libc(msg);
    }
}

#[macro_export]
macro_rules! check_libc {
    ($e:expr) => (check_libc($e, stringify!($e)));
}
