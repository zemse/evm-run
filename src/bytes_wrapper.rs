/// Using Bytes on Clap parses input as UTF-8.
/// Hence, this wrapper is used to parse input as hex.
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct BytesWrapper(revm::primitives::Bytes);

impl std::convert::From<std::string::String> for BytesWrapper {
    fn from(value: std::string::String) -> Self {
        if value.len() % 2 != 0 {
            println!("Invalid hex string: {}", value);
            std::process::exit(1);
        }
        BytesWrapper(revm::primitives::Bytes::from_str(&value).unwrap())
    }
}

impl BytesWrapper {
    pub fn into_inner(self) -> revm::primitives::Bytes {
        self.0
    }
}
