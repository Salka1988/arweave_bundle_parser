use crate::errors::{Error, Result};
use bundlr_sdk::DataItem;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
pub(crate) async fn read_exact_bytes<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    len: usize,
) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

pub(crate) async fn read_u64_le<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    len: usize,
) -> Result<u64> {
    if len != 32 {
        return Err(Error::InvalidDataFormat(format!(
            "Expected 32 bytes for u64, got {} bytes",
            len
        )));
    }
    let buf = read_exact_bytes(reader, len).await?;
    if buf.iter().skip(8).any(|&b| b != 0) {
        return Err(Error::InvalidDataFormat(
            "u64 value exceeds 8 bytes".to_string(),
        ));
    }
    let value = u64::from_le_bytes(
        buf[0..8]
            .try_into()
            .map_err(|_| Error::InvalidDataFormat("Failed to parse u64".to_string()))?,
    );
    Ok(value)
}

pub(crate) async fn read_usize_le<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    len: usize,
) -> Result<usize> {
    let value = read_u64_le(reader, len).await?;
    value
        .try_into()
        .map_err(|_| Error::InvalidDataFormat("usize conversion failed".to_string()))
}

pub async fn parse_and_print_json_file(output_path: &str, print_data: bool) -> Result<()> {
    let file = File::open(output_path).await?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();

    reader.read_to_string(&mut contents).await?;

    let data_items: Vec<DataItem> = serde_json::from_str(&contents)
        .map_err(|e| Error::SerializationError(format!("JSON deserialization failed: {}", e)))?;

    for data_item in data_items {
        println!("{:?}", PrintDataItem::from_data_item(data_item, print_data));
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct PrintDataItem {
    pub signature_type: u8,
    pub signature: String,
    pub owner: String,
    pub target: Option<String>,
    pub anchor: Option<String>,
    pub number_of_tags: u64,
    pub number_of_tag_bytes: u64,
    pub tags: Vec<String>,
    pub data: Vec<u8>,
    #[serde(skip_serializing)]
    pub print_data_flag: bool,
}

impl PrintDataItem {
    pub fn from_data_item(item: DataItem, print_data_flag: bool) -> Self {
        let owner = hex::encode(item.owner);
        let signature = hex::encode(item.signature);
        let target = item.target.map(hex::encode);
        let anchor = item.anchor.map(hex::encode);

        let tags = item
            .tags
            .iter()
            .map(|tag| {
                format!(
                    "Name: {}, Value: {}",
                    String::from_utf8_lossy(&tag.name),
                    String::from_utf8_lossy(&tag.value)
                )
            })
            .collect();

        let number_of_tag_bytes = item
            .tags
            .iter()
            .map(|tag| tag.name.len() + tag.value.len())
            .sum::<usize>() as u64;

        PrintDataItem {
            signature_type: item.signature_type,
            signature,
            owner,
            target,
            anchor,
            number_of_tags: item.tags.len() as u64,
            number_of_tag_bytes,
            tags,
            data: item.data,
            print_data_flag,
        }
    }
}
impl Debug for PrintDataItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Signature Type: {}", self.signature_type)?;
        writeln!(f, "Signature: {}", self.signature)?;
        writeln!(f, "Owner: {}", self.owner)?;
        if let Some(target) = &self.target {
            writeln!(f, "Target: {}", target)?;
        }
        if let Some(anchor) = &self.anchor {
            writeln!(f, "Anchor: {}", anchor)?;
        }
        writeln!(f, "Number of Tags: {}", self.number_of_tags)?;
        writeln!(f, "Number of Tag Bytes: {}", self.number_of_tag_bytes)?;
        writeln!(f, "Tags: ")?;
        for tag in &self.tags {
            writeln!(f, "    {}", tag)?;
        }
        if self.print_data_flag {
            writeln!(f, "Data: {:?}", self.data)?
        }
        Ok(())
    }
}
