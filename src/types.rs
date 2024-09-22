use num_bigint::BigUint;
#[derive(Debug)]
pub struct Bundle {
    pub item_count: BigUint,
    pub offsets: Vec<(BigUint, Vec<u8>)>, // (size, id)
    pub items: Vec<bundlr_sdk::DataItem>,
}