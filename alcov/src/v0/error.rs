use std::ffi::FromBytesUntilNulError;
use std::io;
use std::num::TryFromIntError;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    SizeError(TryFromIntError),
    WrongMagic,
    WrongFlags(u16),
    EmptyModule,
    MalformedBinary,
    PathReadError(FromBytesUntilNulError),
    PathEncodingError,
    DecompressError(lzma_rs::error::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Error::SizeError(err)
    }
}

impl From<FromBytesUntilNulError> for Error {
    fn from(err: FromBytesUntilNulError) -> Self {
        Error::PathReadError(err)
    }
}

impl From<lzma_rs::error::Error> for Error {
    fn from(err: lzma_rs::error::Error) -> Self {
        Error::DecompressError(err)
    }
}
