# ANS-104 Indexer CLI for Arweave


This is a command-line tool to parse and manage Arweave bundles based on the ANS-104 specification. It allows users to fetch a bundle using a transaction ID, parse it, and either print the parsed data to the terminal or save it to a `.json` file.

## Features

- **Print JSON Files**: Print the contents of a JSON file, with an option to include detailed data.
- **Fetch and Parse Bundles**: Fetch a bundle from Arweave using a transaction ID, and either write the parsed data to a file or print it to the terminal.


## Installation

1. Clone this repository:
   ```bash
   git clone git@github.com:Salka1988/arweave_bundle_parser.git
    ```

2. Navigate to the project directory:
   ```bash
    cd arweave_bundle_parser
    ```

3. Build the project:
   ```bash
    cargo build --release
    ```

# Usage

## PrintJson Command
   Print the contents of a specified JSON file.

1. Basic Example:
   ```bash
    arweave_bundle_parser print-json --file path/to/file.json
   ```
   Description: Prints the contents of file.json to the terminal.

2. Including Detailed Data:
    ```bash
     arweave_bundle_parser print-json --file path/to/file.json --print-data
    ```
   Description: Prints the contents of file.json to the terminal, including detailed data content.

## Fetch Command
   Fetch and parse a bundle from Arweave using a transaction ID.

1. Fetch and Write to File (Default):
   ```bash
    arweave_bundle_parser fetch --transaction-id <TRANSACTION_ID> --output path/to/output.json
   ```
    Description: Fetches the bundle using TRANSACTION_ID and writes the parsed data to output.json.

2. Fetch and Print to Terminal:
    ```bash
    arweave_bundle_parser fetch --transaction-id <TRANSACTION_ID> --print
    ```
   Description: Writes contents to file.json and prints them to the terminal

3. Fetch and Print Detailed Data:
    ```bash
   arweave_bundle_parser fetch --transaction-id <TRANSACTION_ID> --print --print-data
    ```
   Description: Fetches the bundle writes it to file.json and prints the parsed transaction data, including detailed data content, to the terminal.

4. Fetch with Custom Output File:
    ```bash
   arweave_bundle_parser fetch --transaction-id <TRANSACTION_ID> --output custom_file.json
    ```
   Description: Fetches the bundle and writes the parsed data to custom_file.json.


# Command Reference
## PrintJson Command
<ol>

`--file`: Path to the JSON file to be printed. Must have a .json extension.

`--print-data`: Optional. If used, prints the data content when printing the file.

</ol>

## Fetch Command
<ol>

`--transaction-id (-t)`: Transaction ID of the Arweave bundle to fetch.

`--output (-o)`: Path to the output JSON file. Must have a .json extension. Defaults to bundle.json.

`--print (-p)`: If used, prints the parsed data to the terminal instead of writing it to a file.

`--print-data (-d)`: Optional. If used with --print, prints the detailed data content.

</ol>