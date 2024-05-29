use crate::{cli, inspector::CustomTracer};
use revm::{
    db::CacheDB,
    inspector_handle_register,
    primitives::{address, keccak256, AccountInfo, ExecutionResult, TransactTo, U256},
    Evm,
};
use std::result::Result;

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
        let code = code.clone().into_inner();
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
            tx.data = calldata.clone().into_inner();
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
