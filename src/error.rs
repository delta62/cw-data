use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Database(rusqlite::Error),
    Environment(&'static str),
    Network(reqwest::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Database(err) => write!(f, "{err}"),
            Error::Environment(var) => write!(f, "{var} is not set"),
            Error::Network(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}
