use std::fmt;
use std::error;
use std::io;

pub type Result<T> = ::std::result::Result<T, Error>;

/// An error that can be produced during (de)serializing.
///
/// If decoding from a Buffer, assume that the buffer has been left
/// in an invalid state.
pub type Error = Box<ErrorKind>;

#[derive(Debug)]
pub enum ErrorKind {
    /// If the error stems from the reader/writer that is being used
    /// during (de)serialization, that error will be stored and returned here.
    IoError(io::Error),
    /// If the bytes in the reader are not decodable because of an invalid
    /// encoding, this error will be returned.  This error is only possible
    /// if a stream is corrupted.  A stream produced from `encode` or `encode_into`
    /// should **never** produce an InvalidEncoding error.
    InvalidEncoding {
        desc: &'static str,
        detail: Option<String>,
    },
    /// If (de)serializing a message takes more than the provided size limit, this
    /// error is returned.
    SizeLimit,
    SequenceMustHaveLength,
    Custom(String),
}

impl error::Error for ErrorKind {
    fn description(&self) -> &str {
        match *self {
            ErrorKind::IoError(ref err) => error::Error::description(err),
            ErrorKind::InvalidEncoding { desc, .. } => desc,
            ErrorKind::SequenceMustHaveLength => "bincode can't encode infinite sequences",
            ErrorKind::SizeLimit => "the size limit for decoding has been reached",
            ErrorKind::Custom(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ErrorKind::IoError(ref err) => err.cause(),
            ErrorKind::InvalidEncoding { .. } => None,
            ErrorKind::SequenceMustHaveLength => None,
            ErrorKind::SizeLimit => None,
            ErrorKind::Custom(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        ErrorKind::IoError(err).into()
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::IoError(ref ioerr) => write!(fmt, "IoError: {}", ioerr),
            ErrorKind::InvalidEncoding { desc, detail: None } => {
                write!(fmt, "InvalidEncoding: {}", desc)
            }
            ErrorKind::InvalidEncoding { desc, detail: Some(ref detail) } => {
                write!(fmt, "InvalidEncoding: {} ({})", desc, detail)
            }
            ErrorKind::SequenceMustHaveLength => {
                write!(fmt,
                       "Bincode can only encode sequences and maps that have a knowable size \
                        ahead of time.")
            }
            ErrorKind::SizeLimit => write!(fmt, "SizeLimit"),
            ErrorKind::Custom(ref s) => s.fmt(fmt),
        }
    }
}

impl ::serde::de::Error for Error {
    fn custom<T: fmt::Display>(desc: T) -> Error {
        ErrorKind::Custom(desc.to_string()).into()
    }
}

impl ::serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ErrorKind::Custom(msg.to_string()).into()
    }
}