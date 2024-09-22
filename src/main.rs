use clap::Parser;
use arweave_bundle_parser::errors::Result;
use arweave_bundle_parser::fetch::fetch_transaction_data;
use arweave_bundle_parser::parse::parse_bundle;
use arweave_bundle_parser::cli::Cli;
use arweave_bundle_parser::utils::parse_and_print_json_file;

#[tokio::main]
async fn main() -> Result<()> {

    let args = Cli::parse();

    let transaction_id = "G7eiK22V-M6RZTcWbq6THzRegvFU6_1NTAHVBOryMpw";
    println!("Fetching transaction data for ID: {}", transaction_id);

    let mut reader = fetch_transaction_data(&args.transaction_id).await?;

    println!("Parsing bundle...");
    parse_bundle(&mut reader, &args.output).await?;
    
    parse_and_print_json_file(&args.output).await?;


    Ok(())
}
// #[cfg(test)]
// mod test {
//     use num_bigint::BigUint;
//     use serde_json::{json, Value};
//     use crate::implementation::parse_bundle;
//     use super::*;
//
//     #[test]
//     fn test_fetch_transaction_data() -> Result<(), ArweaveError> {
//         let transaction_id = "iI6WAayTZB39c0S3kV8yNqXf1TxW7I8poAaOWiEgU18";
//         let data = fetch_transaction_data(transaction_id)?;
//
//         assert_eq!(data.len(), 2924367);
//         Ok(())
//     }
//
//     #[test]
//     fn test_parse_bundle_header() -> Result<(), ArweaveError> {
//         let transaction_id = "iI6WAayTZB39c0S3kV8yNqXf1TxW7I8poAaOWiEgU18";
//         let data = fetch_transaction_data(transaction_id)?;
//         println!("Fetched {} bytes of data.", data.len());
//
//         let bundle = parse_bundle(&data)?;
//         println!("Parsed bundle header successfully!");
//         println!("Number of items: {}", bundle.item_count);
//         assert_eq!(bundle.item_count, BigUint::from(21u64));
//
//         for (i, (size, id)) in bundle.offsets.iter().enumerate() {
//             println!("Item {}: size = {}, id = {}", i + 1, size, hex::encode(id));
//         }
//
//         Ok(())
//     }
//
//     // #[test]
//     // fn test_parse_data_item() -> Result<(), ArweaveError> {
//     //     let transaction_id = "G7eiK22V-M6RZTcWbq6THzRegvFU6_1NTAHVBOryMpw";
//     //     let data = fetch_transaction_data(transaction_id)?;
//     //
//     //     let bundle = parse_bundle(&mut data)?;
//     //     assert_eq!(bundle.item_count, BigUint::from(2u64));
//     //
//     //     let hash = Sha256::digest(&bundle.items[0].owner);
//     //     let from = URL_SAFE_NO_PAD.encode(hash);
//     //     assert_eq!(from, "PEPK6FuFTBrzQdG2fbGLu5vZG-abVA_1m6uqmJaioAM");
//     //
//     //     let hash = Sha256::digest(&bundle.items[0].owner);
//     //     let from = URL_SAFE_NO_PAD.encode(hash);
//     //
//     //     assert_eq!(from, "PEPK6FuFTBrzQdG2fbGLu5vZG-abVA_1m6uqmJaioAM");
//     //     assert_eq!(&bundle.items[0].tags.len(), &10);
//     //
//     //     assert_eq!(bundle.items[0].tags[0].name, b"Drive-Id");
//     //     assert_eq!(bundle.items[0].tags[0].value, b"5b6fb3f0-dd2a-41f6-96bd-aa5755f01f36");
//     //
//     //     assert_eq!(bundle.items[0].tags[1].name, b"Content-Type");
//     //     assert_eq!(bundle.items[0].tags[1].value, b"application/json");
//     //
//     //     assert_eq!(bundle.items[0].tags[2].name, b"ArFS");
//     //     assert_eq!(bundle.items[0].tags[2].value, b"0.14");
//     //
//     //     assert_eq!(bundle.items[0].tags[3].name, b"File-Id");
//     //     assert_eq!(bundle.items[0].tags[3].value, b"7b4796ed-a12c-46ee-8206-7102df326883");
//     //
//     //     assert_eq!(bundle.items[0].tags[4].name, b"Entity-Type");
//     //     assert_eq!(bundle.items[0].tags[4].value, b"file");
//     //
//     //     assert_eq!(bundle.items[0].tags[5].name, b"Parent-Folder-Id");
//     //     assert_eq!(bundle.items[0].tags[5].value, b"76d537ed-3990-4ce4-8768-906aa10b0a5e");
//     //
//     //     assert_eq!(bundle.items[0].tags[6].name, b"App-Name");
//     //     assert_eq!(bundle.items[0].tags[6].value, b"ArDrive-App");
//     //
//     //     assert_eq!(bundle.items[0].tags[7].name, b"App-Platform");
//     //     assert_eq!(bundle.items[0].tags[7].value, b"Web");
//     //
//     //     assert_eq!(bundle.items[0].tags[8].name, b"App-Version");
//     //     assert_eq!(bundle.items[0].tags[8].value, b"2.54.4");
//     //
//     //     assert_eq!(bundle.items[0].tags[9].name, b"Unix-Time");
//     //     assert_eq!(bundle.items[0].tags[9].value, b"1726822174");
//     //
//     //     let json_str =  std::str::from_utf8(&bundle.items[0].data).map_err(|_| ArweaveError::InvalidDataFormat("Invalid UTF-8 data".to_string()))?;
//     //     let real_value = "{\"name\":\"Soft_And_Serene.mp3\",\"size\":2955134,\"lastModifiedDate\":1726821731852,\"dataContentType\":\"audio/mpeg\",\"dataTxId\":\"BEjXfohGQ6sfRpXVprDri_EftGDWmXlAb94JELXVOsY\"}";
//     //     assert_eq!(json_str, real_value);
//     //     assert_eq!(bundle.items[0].data.len(), 166);
//     //
//     //     let hash = Sha256::digest(&bundle.items[1].owner);
//     //     let from = URL_SAFE_NO_PAD.encode(hash);
//     //     assert_eq!(from, "PEPK6FuFTBrzQdG2fbGLu5vZG-abVA_1m6uqmJaioAM");
//     //
//     //     assert_eq!(bundle.items[1].tags[0].name, b"App-Name");
//     //     assert_eq!(bundle.items[1].tags[0].value, b"ArDrive-App");
//     //
//     //     assert_eq!(bundle.items[1].tags[1].name, b"App-Platform");
//     //     assert_eq!(bundle.items[1].tags[1].value, b"Web");
//     //
//     //     assert_eq!(bundle.items[1].tags[2].name, b"App-Version");
//     //     assert_eq!(bundle.items[1].tags[2].value, b"2.54.4");
//     //
//     //     assert_eq!(bundle.items[1].tags[3].name, b"Unix-Time");
//     //     assert_eq!(bundle.items[1].tags[3].value, b"1726822174");
//     //
//     //     assert_eq!(bundle.items[1].tags[4].name, b"Content-Type");
//     //     assert_eq!(bundle.items[1].tags[4].value, b"audio/mpeg");
//     //
//     //     assert_eq!(bundle.items[1].data.len(), 2955134);
//     //
//     //     Ok(())
//     // }
// }
