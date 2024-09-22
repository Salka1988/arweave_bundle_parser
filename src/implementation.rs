use std::io::{Error as IoError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::io::StreamReader;
use bundlr_sdk::{BundlrTx, DataItem};
use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use crate::errors::ArweaveError;
use reqwest::Client;
use serde_json::json;
use tokio::fs::File;
use futures_util::StreamExt;

pub async fn fetch_transaction_data(transaction_id: &str) -> Result<impl tokio::io::AsyncRead + Unpin, ArweaveError> {
    let url = format!("https://arweave.net/{}", transaction_id);
    let client = Client::new();
    let response = client.get(&url).send().await?;
    let stream = response.bytes_stream().map(|result| {
        result.map_err(|e| IoError::new(std::io::ErrorKind::Other, e))
    });
    let reader = StreamReader::new(stream);
    Ok(reader)
}

async fn read_exact_bytes<R: AsyncReadExt + Unpin>(reader: &mut R, len: usize) -> Result<Vec<u8>, ArweaveError> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn read_biguint<R: AsyncReadExt + Unpin>(reader: &mut R, len: usize) -> Result<BigUint, ArweaveError> {
    let buf = read_exact_bytes(reader, len).await?;
    Ok(BigUint::from_bytes_le(&buf))
}

pub async fn parse_bundle<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<(), ArweaveError> {
    // Read the first 32 bytes to get the number of items.
    let item_count = read_biguint(reader, 32).await?;

    // Read offsets.
    let mut offsets = Vec::new();
    for _ in 0..item_count.to_u64().ok_or_else(|| {
        ArweaveError::InvalidDataFormat("Item count too large".to_string())
    })? {
        let size = read_biguint(reader, 32).await?;
        let id = read_exact_bytes(reader, 32).await?;
        offsets.push((size, id));
    }

    // Open a file to write the items as JSON
    let mut file = File::create("bundle.json").await?;

    // Begin the JSON array
    file.write_all(b"[").await?;

    let mut first_item = true;

    // Read DataItems.
    for (size, expected_id) in &offsets {
        let size_usize = size.to_usize().ok_or_else(|| {
            ArweaveError::InvalidDataFormat("Data item size too large".to_string())
        })?;
        let item_data = read_exact_bytes(reader, size_usize).await?;

        let mut data_item = BundlrTx::from_bytes(item_data)
            .map_err(|_| ArweaveError::EncodingError("Failed to parse DataItem".to_string()))?;
        data_item.verify().await.map_err(|_| ArweaveError::EncodingError("Cannot verify".to_string()))?;

        let data_item = DataItem::from(data_item);

        // Convert data_item into JSON
        let item_json = serde_json::to_string(&DataItem::from(data_item))
            .map_err(|_| ArweaveError::EncodingError("JSON serialization failed".to_string()))?;

        // Write to file
        if first_item {
            first_item = false;
        } else {
            file.write_all(b",").await?;
        }

        file.write_all(item_json.as_bytes()).await?;
    }

    // End the JSON array
    file.write_all(b"]").await?;

    Ok(())
}

