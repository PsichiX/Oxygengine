use std::{io::Error as IoError, result::Result as StdResult};

#[derive(Debug)]
pub enum Error {
    Empty,
    Message(String),
}

pub type Result<T> = StdResult<T, Error>;

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Message(format!("{}", error))
    }
}
