use crayon::{res, video};
// use crayon_3d;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Video(video::errors::Error),
    #[fail(display = "{}", _0)]
    Resource(res::errors::Error),
    // #[fail(display = "{}", _0)]
    // Scene(crayon_3d::errors::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<video::errors::Error> for Error {
    fn from(err: video::errors::Error) -> Self {
        Error::Video(err)
    }
}

impl From<res::errors::Error> for Error {
    fn from(err: res::errors::Error) -> Self {
        Error::Resource(err)
    }
}

// impl From<crayon_3d::errors::Error> for Error {
//     fn from(err: crayon_3d::errors::Error) -> Self {
//         Error::Scene(err)
//     }
// }
