use anyhow::Result;
use clap::Parser;
use ethers_providers::{Http, Provider};
use evm_run::cli;
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

    if args.block.is_some() && args.code.is_none() && args.to.is_none() {
        if args.rpc.is_none() {
            println!("Please provide an RPC endpoint to fetch the block data!");
            std::process::exit(1);
        }
        let client = Provider::<Http>::try_from(args.rpc.clone().unwrap())?;
        let client = Arc::new(client);
        let ext_db =
            EthersDB::new(Arc::clone(&client), args.block.map(|n| (n - 1).into())).unwrap();
        println!("running block {}", args.block.unwrap());
        evm_run::block(ext_db, args.block.unwrap(), &args).await;
    } else if let Some(rpc) = &args.rpc {
        let client = Provider::<Http>::try_from(rpc)?;
        let client = Arc::new(client);
        let db: CacheDB<EthersDB<Provider<Http>>> =
            CacheDB::new(EthersDB::new(Arc::clone(&client), args.block.map(|b| b.into())).unwrap());
        evm_run::tx(db, &args)?;
    } else {
        let db = CacheDB::new(EmptyDB::new());
        evm_run::tx(db, &args)?;
    };

    Ok(())
}
