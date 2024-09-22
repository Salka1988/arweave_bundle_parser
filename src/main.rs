use arweave_bundle_parser::cli::{Cli, Commands};
use arweave_bundle_parser::errors::Result;
use arweave_bundle_parser::fetch::fetch_transaction_data;
use arweave_bundle_parser::parse::parse_bundle;
use arweave_bundle_parser::utils::parse_and_print_json_file;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::PrintJson { file, print_data } => {
            parse_and_print_json_file(file.to_str().expect("Json file expected"), print_data)
                .await?;
        }
        Commands::Fetch {
            transaction_id,
            output,
            print,
            print_data,
        } => {
            let mut reader = fetch_transaction_data(&transaction_id).await?;
            let output = output.to_str().expect("Json file expected");

            parse_bundle(&mut reader, output).await?;

            if print {
                parse_and_print_json_file(output, print_data).await?;
            }
        }
    }

    Ok(())
}
