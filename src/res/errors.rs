#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    IO(::std::io::Error),
    #[fail(display = "{}", _0)]
    Bincode(::bincode::Error),
    #[fail(display = "{}", _0)]
    Malformed(String),
    #[fail(display = "{}", _0)]
    Video(::video::errors::Error),
    #[fail(display = "{}", _0)]
    VFS(String),
    #[fail(display = "Location {} is malformed.", _0)]
    MalformLocation(String),
    #[fail(display = "Undefined UUID {}.", _0)]
    UuidNotFound(::utils::uuid::Uuid),
    #[fail(display = "Undefined Path {:?}.", _0)]
    FileNotFound(::std::path::PathBuf),
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

impl From<::bincode::Error> for Error {
    fn from(err: ::bincode::Error) -> Self {
        Error::Bincode(err)
    }
}
