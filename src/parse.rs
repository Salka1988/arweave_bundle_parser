use crate::errors::{Error, Result};
use bundlr_sdk::{BundlrTx, DataItem};
use serde_json::to_string;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};

pub async fn parse_bundle<R: AsyncReadExt + Unpin>(reader: &mut R, output_path: &str) -> Result<()> {
    // Read the first 32 bytes to get the number of items (LittleEndian)
    let item_count = read_u64_le(reader, 32).await?;

    // Read offsets.
    let mut offsets = Vec::with_capacity(item_count as usize);
    for _ in 0..item_count {
        let size = read_usize_le(reader, 32).await?;
        let id = read_exact_bytes(reader, 32).await?;
        offsets.push((size, id));
    }

    // Open a file to write the items as JSON using a buffered writer
    let file = File::create(output_path).await?;
    let mut writer = BufWriter::new(file);

    // Begin the JSON array
    writer.write_all(b"[").await?;

    let mut first_item = true;

    // Read DataItems.
    for (size, _expected_id) in &offsets {
        // Ensure size fits into memory
        let item_data = read_exact_bytes(reader, *size).await?;

        let mut data_item = BundlrTx::from_bytes(item_data)
            .map_err(|e| Error::BundlrSdkError(format!("Failed to parse DataItem: {}", e)))?;
        data_item
            .verify()
            .await
            .map_err(|e| Error::BundlrSdkError(format!("Cannot verify DataItem: {}", e)))?;

        let data_item = DataItem::from(data_item);

        // Convert data_item into JSON
        let item_json = to_string(&data_item)
            .map_err(|e| Error::SerializationError(format!("JSON serialization failed: {}", e)))?;

        // Write to file
        if first_item {
            first_item = false;
        } else {
            writer.write_all(b",").await?;
        }

        writer.write_all(item_json.as_bytes()).await?;
    }

    // End the JSON array
    writer.write_all(b"]").await?;
    writer.flush().await?;

    Ok(())
}

async fn read_exact_bytes<R: AsyncReadExt + Unpin>(reader: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn read_u64_le<R: AsyncReadExt + Unpin>(reader: &mut R, len: usize) -> Result<u64> {
    if len != 32 {
        return Err(Error::InvalidDataFormat(format!(
            "Expected 32 bytes for u64, got {} bytes",
            len
        )));
    }
    let buf = read_exact_bytes(reader, len).await?;
    // Ensure bytes 8-31 are zero
    if buf.iter().skip(8).any(|&b| b != 0) {
        return Err(Error::InvalidDataFormat("u64 value exceeds 8 bytes".to_string()));
    }
    // Read the first 8 bytes as little-endian u64
    let value = u64::from_le_bytes(
        buf[0..8]
            .try_into()
            .map_err(|_| Error::InvalidDataFormat("Failed to parse u64".to_string()))?,
    );
    Ok(value)
}

async fn read_usize_le<R: AsyncReadExt + Unpin>(reader: &mut R, len: usize) -> Result<usize> {
    let value = read_u64_le(reader, len).await?;
    value
        .try_into()
        .map_err(|_| Error::InvalidDataFormat("usize conversion failed".to_string()))
}
