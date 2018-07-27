#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    IO(::std::io::Error),
    #[fail(display = "{}", _0)]
    Compile(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Self {
        Error::IO(err)
    }
}
