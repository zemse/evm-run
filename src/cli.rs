use clap::Parser;
use revm::primitives::{Address, U256};

use crate::bytes_wrapper::BytesWrapper;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub code_positional: Option<BytesWrapper>,
    // DB Arguments
    #[arg(long)]
    pub rpc: Option<String>,
    // Execution Arguments
    #[arg(long)]
    pub code: Option<BytesWrapper>,
    #[arg(long)]
    pub to: Option<Address>,
    // Context Parameters
    #[arg(long)]
    pub from: Option<Address>,
    #[arg(long)]
    pub calldata: Option<BytesWrapper>,
    #[arg(long)]
    pub value: Option<U256>,
    #[arg(long)]
    pub gas: Option<u64>,
}
