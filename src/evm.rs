use crate::{cli, inspector::CustomTracer};
use alloy_provider::{Provider as ProviderTrait, ProviderBuilder};
use ethers_providers::{Http, Provider};
use revm::{
    db::{CacheDB, EthersDB},
    inspector_handle_register,
    primitives::{
        address, calc_blob_gasprice, keccak256, AccountInfo, BlobExcessGasAndPrice, BlockEnv,
        ExecutionResult, TransactTo, TxEnv, U256,
    },
    Evm,
};
use std::{collections::HashMap, result::Result, str::FromStr};

pub fn run<DB>(db: CacheDB<DB>, args: &cli::Args) -> Result<(), anyhow::Error>
where
    CacheDB<DB>: revm::Database + revm::DatabaseCommit,
    <revm::db::CacheDB<DB> as revm::Database>::Error: std::fmt::Debug,
{
    // Step 1: Create an EVM instance
    let evm = Evm::builder()
        .with_db(db)
        .with_external_context(CustomTracer::default())
        .append_handler_register(inspector_handle_register)
        .modify_cfg_env(|f| f.disable_eip3607 = true);

    // Step 2: Set the destination address of the transaction
    let evm = if let Some(to) = &args.to {
        if args.code.is_some() {
            println!("Cannot set both `to` and `code` args at the same time!");
            std::process::exit(1);
        }
        evm.modify_tx_env(|tx| {
            tx.transact_to = TransactTo::Call(*to);
        })
    } else if let Some(code) = &args.code {
        let dummy_address = address!("1234567812345678123456781234567812345678");
        let code = crate::code::parse(code);
        println!("code: {:?}\n", code);
        evm.modify_db(|db| {
            db.insert_account_info(
                dummy_address,
                AccountInfo {
                    balance: U256::ZERO,
                    nonce: 1,
                    code_hash: keccak256(&code),
                    code: Some(revm::primitives::Bytecode::LegacyRaw(code)),
                },
            )
        })
        .modify_tx_env(|tx| {
            tx.transact_to = TransactTo::Call(dummy_address);
        })
    } else {
        println!("Please provide either `to` or `code` args!");
        std::process::exit(1);
    };

    let mut caller = None;

    // Step 3: Set rest of the transaction parameters
    let evm = evm.modify_tx_env(|tx| {
        if let Some(from) = &args.from {
            tx.caller = *from;
        }
        if let Some(calldata) = &args.calldata {
            tx.data = crate::code::parse(calldata);
        }
        if let Some(value) = &args.value {
            tx.value = *value;
        }
        if let Some(gas) = &args.gas {
            tx.gas_limit = *gas;
        }
        caller = Some(tx.caller);
    });

    // Step 4: Modify the account balance if the caller has insufficient balance
    let evm = if let Some(value) = args.value {
        evm.modify_db(|db| {
            let caller = caller.unwrap();
            let mut info = if let Some(db_acc) = db.accounts.get(&caller) {
                db_acc.info.clone()
            } else {
                AccountInfo::default()
            };

            if info.balance < value {
                info.balance = value;
                db.insert_account_info(caller, info);
            }
        })
    } else {
        evm
    };

    // Step 5: Build the EVM instance
    let mut evm = evm.build();

    let initial_gas_spend = evm
        .handler
        .validation()
        .initial_tx_gas(&evm.context.evm.env)
        .unwrap();

    let result = evm.transact_commit().unwrap();

    print_exec_result(result, initial_gas_spend);
    Ok(())
}

pub async fn run_block(
    mut db: CacheDB<EthersDB<Provider<Http>>>,
    block_num: u64,
    args: &cli::Args,
) {
    let provider =
        ProviderBuilder::new().on_http(reqwest::Url::from_str(&args.rpc.clone().unwrap()).unwrap());

    let block = provider
        .get_block(block_num.into(), true)
        .await
        .unwrap()
        .expect("block not found");

    let block_env = BlockEnv {
        number: U256::from(block.header.number.unwrap_or_default()),
        coinbase: block.header.miner,
        timestamp: U256::from(block.header.timestamp),
        gas_limit: U256::from(block.header.gas_limit),
        basefee: U256::from(block.header.base_fee_per_gas.unwrap_or_default()),
        difficulty: block.header.difficulty,
        prevrandao: Some(block.header.difficulty.to_be_bytes::<32>().into()),
        blob_excess_gas_and_price: block.header.excess_blob_gas.map(|excess_blob_gas| {
            let excess_blob_gas = excess_blob_gas as u64;
            BlobExcessGasAndPrice {
                excess_blob_gas,
                blob_gasprice: calc_blob_gasprice(excess_blob_gas),
            }
        }),
    };

    for tx in block.transactions.as_transactions().unwrap() {
        println!("running tx {:?}", tx);
        let mut evm = Evm::builder()
            // .modify_cfg_env(|f| f.disable_eip3607 = true)
            .with_db(db)
            .with_external_context(CustomTracer::default()) // TODO change
            .append_handler_register(inspector_handle_register)
            .with_block_env(block_env.clone())
            .with_tx_env(TxEnv {
                caller: tx.from,
                gas_limit: tx.gas as u64,
                gas_price: U256::from(tx.gas_price.unwrap_or_default()),
                transact_to: match tx.to {
                    Some(addr) => TransactTo::Call(addr),
                    None => TransactTo::Create,
                },
                value: tx.value,
                data: tx.input.clone(),
                nonce: Some(tx.nonce),
                chain_id: tx.chain_id,
                access_list: tx
                    .access_list
                    .clone()
                    .map(|value| {
                        value
                            .0
                            .iter()
                            .map(|item| {
                                (
                                    item.address,
                                    item.storage_keys
                                        .iter()
                                        .map(|k| U256::from_be_slice(k.to_vec().as_slice()))
                                        .collect(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                gas_priority_fee: tx.max_priority_fee_per_gas.map(U256::from),
                blob_hashes: tx.blob_versioned_hashes.clone().unwrap_or_default(),
                max_fee_per_blob_gas: tx.max_fee_per_blob_gas.map(U256::from),
                eof_initcodes: vec![],
                eof_initcodes_hashed: HashMap::default(),
            })
            .build();
        evm.transact_commit().unwrap();
        (db, _) = evm.into_db_and_env_with_handler_cfg();
    }
}

fn print_exec_result(result: ExecutionResult, initial_gas_spend: u64) {
    println!();
    let gas_used = match result {
        ExecutionResult::Success {
            gas_used, output, ..
        } => {
            let data = match output {
                revm::primitives::Output::Call(data) => data,
                revm::primitives::Output::Create(_, _) => unreachable!(),
            };
            println!("Success!\nReturndata: {data}");
            gas_used
        }
        ExecutionResult::Revert { gas_used, output } => {
            println!("Revert!\nRevertdata: {output}");
            gas_used
        }
        ExecutionResult::Halt { reason, gas_used } => {
            println!("Halt!\nReason: {reason:?}");
            gas_used
        }
    };
    println!("Gas used: {}", gas_used - initial_gas_spend);
}
