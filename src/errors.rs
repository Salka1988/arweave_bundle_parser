use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpRequestError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),

    #[error("Encoding/Decoding error: {0}")]
    EncodingError(String),

    #[error("Bundlr SDK error: {0}")]
    BundlrSdkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
