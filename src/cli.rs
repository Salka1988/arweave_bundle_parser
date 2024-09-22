use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The transaction ID of the Arweave bundle to parse
    #[arg(short, long)]
    pub transaction_id: String,
    /// The output JSON file path (optional, defaults to 'bundle.json')
    #[arg(short, long)]
    pub output: String,
}