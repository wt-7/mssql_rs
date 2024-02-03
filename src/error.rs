pub type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Tiberius(#[from] tiberius::error::Error),
    #[error("Connection to database timed out")]
    ConnectionTimeout,
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No results found")]
    EmptyResult,
}

impl From<bb8::RunError<Error>> for Error {
    fn from(error: bb8::RunError<Error>) -> Self {
        match error {
            bb8::RunError::User(e) => e,
            bb8::RunError::TimedOut => Error::ConnectionTimeout,
        }
    }
}
