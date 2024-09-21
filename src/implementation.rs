use std::io::{Cursor, Read};
use bundlr_sdk::BundlrTx;
use bytes::{Buf, Bytes};
use num_bigint::BigUint;
use num_traits::{ToBytes, ToPrimitive};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use crate::errors::ArweaveError;
use crate::types::{Bundle, DataItem, Tag};

/// Function to fetch the transaction data from Arweave network.
pub fn fetch_transaction_data(transaction_id: &str) -> Result<Bytes, ArweaveError> {
    let url = format!("https://arweave.net/{}", transaction_id);
    let client = Client::new();
    let response = client.get(&url).send()?;
    let bytes = response.bytes()?;
    Ok(bytes)
}


pub fn read_u256(cursor: &mut Cursor<&[u8]>) -> Result<BigUint, ArweaveError> {
    let mut buf = [0u8; 32];
    cursor.read_exact(&mut buf)?;
    Ok(BigUint::from_bytes_le(&buf))
}

pub fn parse_bundle(data: &[u8]) -> Result<Bundle, ArweaveError> {
    let mut cursor = Cursor::new(data);

    // Read the first 32 bytes to get the number of items.
    let item_count = read_u256(&mut cursor)?;

    // Validate item count does not exceed a reasonable limit (optional)
    if item_count > BigUint::from(10000u32) {
        return Err(ArweaveError::InvalidDataFormat(
            "Item count too large".to_string(),
        ));
    }

    // Read offsets.
    let mut offsets = Vec::new();
    for _ in 0..item_count
        .to_u64()
        .ok_or_else(|| ArweaveError::InvalidDataFormat("Item count too large".to_string()))?
    {
        let size = read_u256(&mut cursor)?;
        let mut id = vec![0u8; 32];
        cursor.read_exact(&mut id)?;
        offsets.push((size, id));
    }

    // Read DataItems.
    let mut items = Vec::new();
    for (size, expected_id) in &offsets {
        let size_usize = size.to_usize().ok_or_else(|| {
            ArweaveError::InvalidDataFormat("Data item size too large".to_string())
        })?;
        let mut item_data = vec![0u8; size_usize];
        cursor.read_exact(&mut item_data)?;

        // Parse the DataItem
        let data_item = parse_data_item(&item_data)?;
        // let data_item = BundlrTx::from_bytes(item_data)?;
        // let data_item = DataItem::deserialize(item_data).map_err(|e| ArweaveError::EncodingError("arst".to_string()))?;


        // Verify DataItem ID matches expected ID
        // let computed_id = Sha256::digest(data_item.signature_type.to_be_bytes());
        // if &computed_id[..] != &expected_id[..] {
        //     return Err(ArweaveError::InvalidDataFormat(format!(
        //         "DataItem ID mismatch. Expected: {}, Computed: {}",
        //         hex::encode(expected_id),
        //         hex::encode(computed_id)
        //     )));
        // }

        items.push(data_item);
    }

    Ok(Bundle {
        item_count,
        offsets,
        items,
    })
}

/// Function to parse a DataItem from binary data.
pub fn parse_data_item(data: &[u8]) -> Result<DataItem, ArweaveError> {
    let mut cursor = Cursor::new(data);

    // Read signature type (2 bytes)
    if cursor.remaining() < 2 {
        return Err(ArweaveError::InvalidDataFormat(
            "Insufficient data for signature type".to_string(),
        ));
    }
    let signature_type = cursor.get_u16_le();

    // Read signature (depends on signature type)
    let signature_length = get_signature_length(signature_type)?;
    let mut signature = vec![0u8; signature_length];
    cursor.read_exact(&mut signature)?;

    // Read owner (depends on signature type)
    let owner_length = get_owner_length(signature_type)?;
    let mut owner = vec![0u8; owner_length];
    cursor.read_exact(&mut owner)?;

    // Read target (optional)
    let target = read_optional_field(&mut cursor)?;

    // Read anchor (optional)
    let anchor = read_optional_field(&mut cursor)?;

    // Validate anchor length
    if let Some(anchor) = &anchor {
        if anchor.len() > 32 {
            return Err(ArweaveError::InvalidDataFormat(
                "Anchor length exceeds 32 bytes".to_string(),
            ));
        }
    }

    // Read number of tags (8 bytes)
    if cursor.remaining() < 8 {
        return Err(ArweaveError::InvalidDataFormat(
            "Insufficient data for number of tags".to_string(),
        ));
    }
    let number_of_tags = cursor.get_u64_le();

    // Read number of tag bytes (8 bytes)
    if cursor.remaining() < 8 {
        return Err(ArweaveError::InvalidDataFormat(
            "Insufficient data for number of tag bytes".to_string(),
        ));
    }
    let number_of_tag_bytes = cursor.get_u64_le();

    // Read tags
    let mut tags_data = vec![0u8; number_of_tag_bytes as usize];
    cursor.read_exact(&mut tags_data)?;
    let tags = parse_tags(&tags_data)?;

    // Validate number of tags
    if tags.len() != number_of_tags as usize {
        return Err(ArweaveError::InvalidDataFormat(
            "Number of tags does not match the specified count".to_string(),
        ));
    }

    // Validate tag count does not exceed 128
    if tags.len() > 128 {
        return Err(ArweaveError::InvalidDataFormat(
            "Number of tags exceeds 128".to_string(),
        ));
    }

    // Read data (remaining bytes)
    let mut data = Vec::new();
    cursor.read_to_end(&mut data)?;

    // // Construct the DataItem
    let data_item = DataItem {
        signature_type,
        signature: signature.clone(),
        owner: owner.clone(),
        target,
        anchor,
        number_of_tags,
        number_of_tag_bytes,
        tags,
        data: data.clone(),
    };

    // Verify the signature matches the owner's public key
    // verify_signature(&data_item)?;
    // use arloader::crypto::Provider;

    // let provider = Provider::new();
    // DeepHashItem::from_item(&data_item);
    // provider.deep_hash(&data_item);

    Ok(data_item)

}


fn signature(data_item: &DataItem) -> Result<(), ArweaveError> {
    // Compute the deep hash of the DataItem
    let message = deep_hash_data_item(data_item)?;

    // Verify the signature using the appropriate cryptographic method
    // This is a placeholder function; you should implement the actual verification
    if !crypto_verify_signature(
        data_item.signature_type,
        &data_item.owner,
        &data_item.signature,
        &message,
    ) {
        return Err(ArweaveError::InvalidDataFormat(
            "Signature verification failed".to_string(),
        ));
    }

    Ok(())
}

// Placeholder function for signature verification.
fn crypto_verify_signature(
    signature_type: u16,
    owner: &[u8],
    signature: &[u8],
    message: &[u8],
) -> bool {
    match signature_type {
        1 => {
            true // Placeholder for actual verification
        }
        _ => false, // Unsupported signature type
    }
}

fn deep_hash_data_item(data_item: &DataItem) -> Result<Vec<u8>, ArweaveError> {
    // Construct the message to hash as per the specification
    // [
    //   utf8Encoded("dataitem"),
    //   utf8Encoded("1"),
    //   owner,
    //   target,
    //   anchor,
    //   [
    //     ... [ tag.name, tag.value ],
    //     ... [ tag.name, tag.value ],
    //     ...
    //   ],
    //   data
    // ]
    let mut chunks = Vec::new();
    chunks.push(b"dataitem".to_vec());
    chunks.push(b"1".to_vec());
    chunks.push(data_item.owner.clone());

    // Handle optional target and anchor
    if let Some(target) = &data_item.target {
        chunks.push(target.clone());
    } else {
        chunks.push(vec![]);
    }

    if let Some(anchor) = &data_item.anchor {
        chunks.push(anchor.clone());
    } else {
        chunks.push(vec![]);
    }

    // Handle tags
    let mut tags_chunks = Vec::new();
    for tag in &data_item.tags {
        tags_chunks.push(tag.name.clone());
        tags_chunks.push(tag.value.clone());
    }
    chunks.push(deep_hash_chunks(&tags_chunks)?);

    chunks.push(data_item.data.clone());

    // Compute the deep hash
    let message = deep_hash_chunks(&chunks)?;

    Ok(message)
}

fn deep_hash_chunks(chunks: &[Vec<u8>]) -> Result<Vec<u8>, ArweaveError> {
    if chunks.len() == 1 {
        return Ok(Sha256::digest(&chunks[0]).to_vec());
    }

    let mut hasher = Sha256::new();
    for chunk in chunks {
        let hash = Sha256::digest(&chunk).to_vec();
        hasher.update(&hash);
    }
    Ok(hasher.finalize().to_vec())
}

// Function to parse tags using manual decoding as per specification.
pub fn parse_tags(data: &[u8]) -> Result<Vec<Tag>, ArweaveError> {
    let mut cursor = Cursor::new(data);
    let mut tags = Vec::new();

    loop {
        // Read block count (ZigZag VInt)
        if !cursor.has_remaining() {
            break;
        }
        let count = read_zigzag_vint(&mut cursor)?;
        if count == 0 {
            // End of tags
            break;
        }

        let abs_count = count.abs() as usize;

        // If count is negative, read size (ZigZag VInt)
        if count < 0 {
            let _size = read_zigzag_vint(&mut cursor)?;
            // In this context, size is not used directly
        }

        // Read each tag item
        for _ in 0..abs_count {
            // Read name
            let name = read_bytes(&mut cursor)?;
            if name.is_empty() || name.len() > 1024 {
                return Err(ArweaveError::InvalidDataFormat(
                    "Tag name is invalid".to_string(),
                ));
            }

            // Read value
            let value = read_bytes(&mut cursor)?;
            if value.is_empty() || value.len() > 3072 {
                return Err(ArweaveError::InvalidDataFormat(
                    "Tag value is invalid".to_string(),
                ));
            }

            tags.push(Tag { name, value });
        }
    }

    Ok(tags)
}

// Function to read bytes prefixed with ZigZag VInt length.
fn read_bytes(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>, ArweaveError> {
    let length = read_zigzag_vint(cursor)?;
    if length < 0 {
        return Err(ArweaveError::InvalidDataFormat(
            "Negative length for bytes".to_string(),
        ));
    }
    let length = length as usize;

    // Check for potential overflow
    if cursor.remaining() < length {
        return Err(ArweaveError::InvalidDataFormat(
            "Insufficient data for reading bytes".to_string(),
        ));
    }

    let mut buf = vec![0u8; length];
    cursor.read_exact(&mut buf)?;
    Ok(buf)
}


/// Function to read a ZigZag VInt.
fn read_zigzag_vint(cursor: &mut Cursor<&[u8]>) -> Result<i64, ArweaveError> {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        if !cursor.has_remaining() {
            return Err(ArweaveError::EncodingError(
                "Unexpected end of data while reading ZigZag VInt".to_string(),
            ));
        }
        let byte = cursor.get_u8();
        result |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift > 64 {
            return Err(ArweaveError::EncodingError(
                "ZigZag VInt is too long".to_string(),
            ));
        }
    }
    let zigzag = result;
    let value = ((zigzag >> 1) as i64) ^ (-((zigzag & 1) as i64));
    Ok(value)
}

// Helper functions will be added in the next steps.

/// Helper function to get signature length based on signature type.
fn get_signature_length(signature_type: u16) -> Result<usize, ArweaveError> {
    match signature_type {
        1 => Ok(512),
        _ => Err(ArweaveError::InvalidDataFormat(format!(
            "Unknown signature type: {}",
            signature_type
        ))),
    }
}

/// Helper function to get owner length based on signature type.
fn get_owner_length(signature_type: u16) -> Result<usize, ArweaveError> {
    match signature_type {
        1 => Ok(512),
        // Update with actual lengths per signature type
        _ => Err(ArweaveError::InvalidDataFormat(format!(
            "Unknown signature type: {}",
            signature_type
        ))),
    }
}

/// Function to read optional fields (target and anchor).
fn read_optional_field(cursor: &mut Cursor<&[u8]>) -> Result<Option<Vec<u8>>, ArweaveError> {
    let present = cursor.get_u8();
    if present == 1 {
        let mut buf = vec![0u8; 32];
        cursor.read_exact(&mut buf)?;
        Ok(Some(buf))
    } else if present == 0 {
        Ok(None)
    } else {
        Err(ArweaveError::InvalidDataFormat(
            "Invalid presence byte for optional field".to_string(),
        ))
    }
}