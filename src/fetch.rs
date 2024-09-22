use crate::errors::{Error, Result};
use futures_util::StreamExt;
use reqwest::Client;
use std::io::Error as IoError;
use tokio_util::io::StreamReader;

pub async fn fetch_transaction_data(
    transaction_id: &str,
) -> Result<impl tokio::io::AsyncRead + Unpin> {
    let url = format!("https://arweave.net/{}", transaction_id);
    let client = Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| Error::HttpRequestError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(Error::HttpRequestError(format!(
            "{}: {}",
            reqwest::StatusCode::from_u16(response.status().as_u16())
                .expect("Invalid status code encountered"),
            "Failed to fetch transaction data",
        )));
    }

    let stream = response
        .bytes_stream()
        .map(|result| result.map_err(|e| IoError::new(std::io::ErrorKind::Other, e)));
    let reader = StreamReader::new(stream);
    Ok(reader)
}
