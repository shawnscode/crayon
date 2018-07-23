#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    IO(::std::io::Error),
    #[fail(display = "{}", _0)]
    Malformed(String),
    #[fail(display = "{}", _0)]
    Video(::video::errors::Error),
    #[fail(display = "Undefined resource schema {}", _0)]
    Undefined(&'static str),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<::video::errors::Error> for Error {
    fn from(err: ::video::errors::Error) -> Self {
        Error::Video(err)
    }
}
