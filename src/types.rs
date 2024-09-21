use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub name: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug)]
pub struct DataItem {
    pub signature_type: u16,
    pub signature: Vec<u8>,
    pub owner: Vec<u8>,
    pub target: Option<Vec<u8>>,
    pub anchor: Option<Vec<u8>>,
    pub number_of_tags: u64,
    pub number_of_tag_bytes: u64,
    pub tags: Vec<Tag>,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Bundle {
    pub item_count: BigUint,
    pub offsets: Vec<(BigUint, Vec<u8>)>, // (size, id)
    pub items: Vec<DataItem>,
}