use std::io::{Cursor, Read};

use bytes::{Buf, Bytes};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use reqwest::blocking::Client;
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

    // Read offsets.
    let mut offsets = Vec::new();
    for _ in 0..item_count.to_u64().ok_or_else(|| ArweaveError::InvalidDataFormat("Item count too large".to_string()))? {
        let size = read_u256(&mut cursor)?;
        let mut id = vec![0u8; 32];
        cursor.read_exact(&mut id)?;
        offsets.push((size, id));
    }

    // Read DataItems.
    let mut items = Vec::new();
    for (size, _) in &offsets {
        let size_usize = size.to_usize().ok_or_else(|| ArweaveError::InvalidDataFormat("Data item size too large".to_string()))?;
        let mut item_data = vec![0u8; size_usize];
        cursor.read_exact(&mut item_data)?;
        let data_item = parse_data_item(&item_data)?;
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
    let signature_type = cursor.get_u16_le();;

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

    // Read number of tags (8 bytes)
    let number_of_tags = cursor.get_u64_le();

    // Read number of tag bytes (8 bytes)
    let number_of_tag_bytes = cursor.get_u64_le();

    // Read tags
    let mut tags_data = vec![0u8; number_of_tag_bytes as usize];
    cursor.read_exact(&mut tags_data)?;
    let tags = parse_tags(&tags_data)?;

    // tags.iter().for_each(|tag| {
    //     println!("Tag: {} = {}", String::from_utf8_lossy(&tag.name), String::from_utf8_lossy(&tag.value));
    // });

    // Read data (remaining bytes)
    let mut data = Vec::new();
    cursor.read_to_end(&mut data)?;

    Ok(DataItem {
        signature_type,
        signature,
        owner,
        target,
        anchor,
        number_of_tags,
        number_of_tag_bytes,
        tags,
        data,
    })
}
/// Function to parse tags using manual decoding as per specification.
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
            // Read value
            let value = read_bytes(&mut cursor)?;

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
        2 => Ok(512),
        3 => Ok(512),
        // Update with actual lengths per signature type
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
        2 => Ok(512),
        3 => Ok(512),
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



//
// /// Function to parse a DataItem from binary data.
// pub fn parse_data_item(data: &[u8]) -> Result<DataItem, ArweaveError> {
//     let mut cursor = Cursor::new(data);
//
//     // Read signature type (2 bytes, big-endian)
//     let signature_type = cursor.get_u64_le();
//
//     // Read signature (depends on signature type)
//     let signature_length = get_signature_length(signature_type as u16)?;
//     let mut signature = vec![0u8; signature_length];
//     cursor.read_exact(&mut signature)?;
//
//     // Read owner (depends on signature type)
//     let owner_length = get_owner_length(signature_type as u16)?;
//     let mut owner = vec![0u8; owner_length];
//     cursor.read_exact(&mut owner)?;
//
//     // Read target (optional)
//     let target = read_optional_field(&mut cursor)?;
//
//     // Read anchor (optional)
//     let anchor = read_optional_field(&mut cursor)?;
//
//     // Read number of tags (8 bytes, big-endian)
//     let num_of_tags = cursor.get_u64_le();
//
//     // Read number of tag bytes (8 bytes, big-endian)
//     let tags_bytes_length = cursor.get_u64_le();
//
//     // Read tags
//     let mut tags_data = vec![0u8; tags_bytes_length as usize];
//     cursor.read_exact(&mut tags_data)?;
//     let tags = parse_tags(&tags_data)?;
//
//     // Read data (remaining bytes)
//     let mut data = Vec::new();
//     cursor.read_to_end(&mut data)?;
//
//     Ok(DataItem {
//         signature_type: signature_type as u16,
//         signature,
//         owner,
//         target,
//         anchor,
//         tags,
//         data,
//     })
// }
//
// /// Function to parse tags using manual decoding as per specification.
// pub fn parse_tags(data: &[u8]) -> Result<Vec<Tag>, ArweaveError> {
//     let mut cursor = Cursor::new(data);
//     let mut tags = Vec::new();
//
//     loop {
//         // Read block count (ZigZag VInt)
//         let count = read_zigzag_vint(&mut cursor)?;
//         if count == 0 {
//             // End of tags
//             break;
//         }
//
//         let abs_count = count.abs() as usize;
//
//         // If count is negative, read size (ZigZag VInt)
//         if count < 0 {
//             let _size = read_zigzag_vint(&mut cursor)?;
//             // In this context, size is not used directly
//         }
//
//         // Read each tag item
//         for _ in 0..abs_count {
//             // Read name
//             let name = read_bytes(&mut cursor)?;
//             // Read value
//             let value = read_bytes(&mut cursor)?;
//
//             tags.push(Tag { name, value });
//         }
//     }
//
//     Ok(tags)
// }
//
// /// Function to read bytes prefixed with ZigZag VInt length.
// fn read_bytes(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>, ArweaveError> {
//     let length = read_zigzag_vint(cursor)?;
//     if length < 0 {
//         return Err(ArweaveError::InvalidDataFormat(
//             "Negative length for bytes".to_string(),
//         ));
//     }
//     let length = length as usize;
//     let mut buf = vec![0u8; length];
//     cursor.read_exact(&mut buf)?;
//     Ok(buf)
// }
//
// /// Function to read optional fields (target and anchor).
// fn read_optional_field(cursor: &mut Cursor<&[u8]>) -> Result<Option<Vec<u8>>, ArweaveError> {
//     let present = cursor.get_u8();
//     if present == 1 {
//         let mut buf = vec![0u8; 32];
//         cursor.read_exact(&mut buf)?;
//         Ok(Some(buf))
//     } else if present == 0 {
//         Ok(None)
//     } else {
//         Err(ArweaveError::InvalidDataFormat(
//             "Invalid presence byte for optional field".to_string(),
//         ))
//     }
// }
//
// /// Function to read a ZigZag VInt.
// fn read_zigzag_vint(cursor: &mut Cursor<&[u8]>) -> Result<i64, ArweaveError> {
//     let mut result = 0u64;
//     let mut shift = 0;
//     loop {
//         if !cursor.has_remaining() {
//             return Err(ArweaveError::EncodingError(
//                 "Unexpected end of data while reading ZigZag VInt".to_string(),
//             ));
//         }
//         let byte = cursor.get_u8();
//         result |= ((byte & 0x7F) as u64) << shift;
//         if (byte & 0x80) == 0 {
//             break;
//         }
//         shift += 7;
//         if shift > 64 {
//             return Err(ArweaveError::EncodingError(
//                 "ZigZag VInt is too long".to_string(),
//             ));
//         }
//     }
//     let zigzag = result;
//     let value = ((zigzag >> 1) as i64) ^ (-((zigzag & 1) as i64));
//     Ok(value)
// }
//
// /// Helper function to get signature length based on signature type.
// fn get_signature_length(signature_type: u16) -> Result<usize, ArweaveError> {
//     match signature_type {
//         1 => Ok(512),
//         2 => Ok(512),
//         3 => Ok(512),
//         // Add other signature types and their lengths as per specification
//         _ => Err(ArweaveError::InvalidDataFormat(format!(
//             "Unknown signature type: {}",
//             signature_type
//         ))),
//     }
// }
//
// /// Helper function to get owner length based on signature type.
// fn get_owner_length(signature_type: u16) -> Result<usize, ArweaveError> {
//     match signature_type {
//         1 => Ok(512),
//         2 => Ok(512),
//         3 => Ok(512),
//         // Add other signature types and their owner lengths as per specification
//         _ => Err(ArweaveError::InvalidDataFormat(format!(
//             "Unknown signature type: {}",
//             signature_type
//         ))),
//     }
// }

