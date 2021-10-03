//! Error and Result types.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("http status {0}")]
    HttpStatus(u16),
    #[error("http error `{0}`")]
    HttpError(String),
}

pub type Result<T> = std::result::Result<T, HttpError>;
