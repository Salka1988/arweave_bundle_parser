use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum ArweaveError {
    #[error("HTTP request failed: {0}")]
    HttpRequestError(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),
    #[error("Encoding error: {0}")]
    EncodingError(String),
}

impl From<reqwest::Error> for ArweaveError {
    fn from(err: reqwest::Error) -> Self {
        ArweaveError::HttpRequestError(err.to_string())
    }
}

impl From<io::Error> for ArweaveError {
    fn from(err: io::Error) -> Self {
        ArweaveError::IoError(err.to_string())
    }
}