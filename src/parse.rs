use crate::errors::{Error, Result};
use bundlr_sdk::{BundlrTx, DataItem};
use serde_json::to_string;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use crate::utils::{read_exact_bytes, read_u64_le, read_usize_le};

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