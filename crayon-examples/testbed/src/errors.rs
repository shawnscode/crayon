use crayon::{res, video};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Video(video::errors::Error),
    #[fail(display = "{}", _0)]
    Res(res::errors::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<video::errors::Error> for Error {
    fn from(err: video::errors::Error) -> Self {
        Error::Video(err)
    }
}

impl From<res::errors::Error> for Error {
    fn from(err: res::errors::Error) -> Self {
        Error::Res(err)
    }
}
