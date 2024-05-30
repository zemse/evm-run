use clap::Parser;
use revm::primitives::{Address, U256};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub code_positional: Option<String>,
    // DB Arguments
    #[arg(long)]
    pub rpc: Option<String>,
    // Execution Arguments
    #[arg(long)]
    pub code: Option<String>,
    #[arg(long)]
    pub to: Option<Address>,
    // Context Parameters
    #[arg(long)]
    pub from: Option<Address>,
    #[arg(long)]
    pub calldata: Option<String>,
    #[arg(long)]
    pub value: Option<U256>,
    #[arg(long)]
    pub gas: Option<u64>,
}
