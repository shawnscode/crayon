pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

macro_rules! err_format {
    ($e:expr) => {
        $crate::failure::err_msg($e);
    };
    ($fmt:expr, $($arg:tt)+) => {
        $crate::failure::err_msg(format!($fmt, $($arg)+));
    };
}
