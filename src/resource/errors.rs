#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)] IO(::std::io::Error),
    #[fail(display = "{}", _0)] Zip(::zip::result::ZipError),
    #[fail(display = "Drive identifier is duplicated.")] DriveIdentDuplicated,
    #[fail(display = "Failed to find drive with identifier \'{}\'", _0)] DriveNotFound(String),
    #[fail(display = "Failed to find filesystem at {}", _0)] FilesystemNotFound(String),
    #[fail(display = "Failed to find file at {}", _0)] FileNotFound(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<::zip::result::ZipError> for Error {
    fn from(err: ::zip::result::ZipError) -> Self {
        Error::Zip(err)
    }
}
