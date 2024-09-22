use clap::{Parser, Subcommand};
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author = "Your Name <your.email@example.com>",
    version = "1.0.0",
    about = "Parses Arweave bundles and manages JSON files",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    PrintJson {
        #[arg(
            value_name = "FILE",
            value_parser = |s: &str| -> Result<PathBuf, String> {
            let path = PathBuf::from(s);
            if path.extension() == Some(OsStr::new("json")) {
            Ok(path)
            } else {
            Err(String::from("The input file must be bundles file and must have have a .json extension"))
            }
            },
            help = "Path to the JSON file you want to print. Must have a .json extension."
        )]
        file: PathBuf,

        #[arg(
            long = "print-data",
            short = 'd',
            default_value_t = false,
            help = "Include detailed data content when printing the JSON file."
        )]
        print_data: bool,
    },

    Fetch {
        #[arg(
            short = 't',
            long = "transaction-id",
            value_name = "TX_ID",
            help = "The Transaction ID of the Arweave bundle you want to fetch and parse."
        )]
        transaction_id: String,

        #[arg(
            short = 'o',
            long = "output",
            value_name = "OUTPUT",
            default_value = "bundle.json",
            value_parser = |s: &str| -> Result<PathBuf, String> {
            let path = PathBuf::from(s);
            if path.extension() == Some(OsStr::new("json")) {
            Ok(path)
            } else {
            Err(String::from("The output file must have a .json extension"))
            }
            },
            help = "Path to the output JSON file. Defaults to 'bundle.json'. Must have a .json extension."
        )]
        output: PathBuf,

        #[arg(
            long = "print",
            short = 'p',
            default_value_t = false,
            help = "Print the parsed transaction data to the terminal."
        )]
        print: bool,

        #[arg(
            long = "print-data",
            short = 'd',
            requires = "print",
            default_value_t = false,
            help = "Include detailed data content when printing transaction data."
        )]
        print_data: bool,
    },
}
