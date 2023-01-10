use serde::de;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Syntax,
    InvalidHex,
    TrailingCharacters,
    UnexpectedEof,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
