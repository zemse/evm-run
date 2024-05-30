mod cli;
mod code;
mod evm;
mod inspector;
mod stack_fmt;

use anyhow::Result;
use clap::Parser;
use ethers_providers::{Http, Provider};
use revm::db::{CacheDB, EmptyDB, EthersDB};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = cli::Args::parse();
    if let Some(code_positional) = &args.code_positional {
        if args.code.is_some() {
            println!("Cannot have a positional argument when `--code` is used!");
            std::process::exit(1);
        }
        args.code = Some(code_positional.clone());
    }

    if let Some(rpc) = &args.rpc {
        let client = Provider::<Http>::try_from(rpc)?;
        let client = Arc::new(client);
        let db = CacheDB::new(EthersDB::new(Arc::clone(&client), None).unwrap());
        evm::run(db, &args)?;
    } else {
        let db = CacheDB::new(EmptyDB::new());
        evm::run(db, &args)?;
    };

    Ok(())
}
