# WikiReader

WikiReader is a Rust-based command-line application designed to fetch and process Wikipedia articles. It leverages several libraries to provide a seamless experience for users.

## Features

-   Fetch Wikipedia articles using `reqwest`.
-   Parse and process HTML content with `soup`.
-   Serialize and deserialize configurations using `serde`.
-   Generate random words with `random_word`.
-   Display colorful output using `colored`.

## Installation

To build and run WikiReader, ensure you have [Rust](https://www.rust-lang.org/) installed. Then, clone the repository and build the project:

```bash
# Clone the repository
git clone https://github.com/Arman511/wikireader.git
cd wikireader

# Build the project
cargo build --release

# Run the project
cargo run
```

## Dependencies

The project uses the following dependencies:

-   [confy](https://crates.io/crates/confy): For configuration management.
-   [reqwest](https://crates.io/crates/reqwest): For making HTTP requests.
-   [serde](https://crates.io/crates/serde): For serialization and deserialization.
-   [serde_json](https://crates.io/crates/serde_json): For working with JSON data.
-   [random_word](https://crates.io/crates/random_word): For generating random words.
-   [soup](https://crates.io/crates/soup): For HTML parsing.
-   [colored](https://crates.io/crates/colored): For colorful terminal output.

## Usage

The application fetches a Wikipedia article based on a default or user-specified configuration. The default article is "Cheese." You can modify the configuration by editing the `SessionConfig` struct in the code.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
