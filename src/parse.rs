// src/parse.rs

use crate::errors::{Error, Result};
use bundlr_sdk::{BundlrTx, DataItem};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde_json::to_string;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn parse_bundle<R: AsyncReadExt + Unpin>(reader: &mut R, output_path: &str) -> Result<()> {
    // Read the first 32 bytes to get the number of items (BigEndian)
    let item_count = read_biguint(reader, 32).await?;

    // Read offsets.
    let mut offsets = Vec::new();
    for _ in 0..item_count
        .to_u64()
        .ok_or_else(|| Error::InvalidDataFormat("Item count too large".to_string()))?
    {
        let size = read_biguint(reader, 32).await?;
        let id = read_exact_bytes(reader, 32).await?;
        offsets.push((size, id));
    }

    // Open a file to write the items as JSON
    let mut file = File::create(output_path).await?;

    // Begin the JSON array
    file.write_all(b"[").await?;

    let mut first_item = true;

    // Read DataItems.
    for (size, expected_id) in &offsets {
        let size_usize = size
            .to_usize()
            .ok_or_else(|| Error::InvalidDataFormat("Data item size too large".to_string()))?;
        let item_data = read_exact_bytes(reader, size_usize).await?;

        let mut data_item = BundlrTx::from_bytes(item_data)
            .map_err(|e| Error::BundlrSdkError(format!("Failed to parse DataItem: {}", e)))?;
        data_item
            .verify()
            .await
            .map_err(|e| Error::BundlrSdkError(format!("Cannot verify: {}", e)))?;

        let data_item = DataItem::from(data_item);

        // Convert data_item into JSON
        let item_json = to_string(&data_item)
            .map_err(|e| Error::SerializationError(format!("JSON serialization failed: {}", e)))?;

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

async fn read_exact_bytes<R: AsyncReadExt + Unpin>(reader: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn read_biguint<R: AsyncReadExt + Unpin>(reader: &mut R, len: usize) -> Result<BigUint> {
    let buf = read_exact_bytes(reader, len).await?;
    Ok(BigUint::from_bytes_le(&buf))
}
