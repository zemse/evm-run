use crate::inspector::{CustomTracer, CustomTracerResult};
use alloy_provider::{Provider as ProviderTrait, ProviderBuilder};
use indicatif::ProgressBar;
use revm::{
    db::{CacheDB, DbAccount},
    inspector_handle_register,
    primitives::{
        calc_blob_gasprice, Account, AccountInfo, Address, BlobExcessGasAndPrice, BlockEnv,
        Bytecode, TransactTo, TxEnv, B256, U256,
    },
    Database, DatabaseCommit, DatabaseRef, Evm,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

pub async fn run_block<ExtDB>(
    db: ExtDB,
    block_num: u64,
    rpc: &str,
) -> (BlockEnv, Vec<TxEnv>, CacheDB<VoidDB>)
where
    ExtDB: DatabaseRef,
    <ExtDB as revm::DatabaseRef>::Error: std::fmt::Debug,
{
    let mut db = RecorderDB::new(db, format!("block_{block_num}.db"));

    let provider = ProviderBuilder::new().on_http(reqwest::Url::from_str(rpc).unwrap());

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

    let progress_bar = ProgressBar::new(block.header.gas_used as u64);

    let mut tx_vec = vec![];

    for tx in block.transactions.as_transactions().unwrap_or_default() {
        let tx_env = TxEnv {
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
        };
        tx_vec.push(tx_env.clone());

        // println!("running tx {:?} {:?} {}", tx.hash, tx.from, tx.nonce);
        let mut tx_outcome = CustomTracerResult::default();
        let mut evm = Evm::builder()
            // .modify_cfg_env(|f| f.disable_eip3607 = true)
            .with_db(db)
            .with_external_context(CustomTracer::new(&mut tx_outcome)) // TODO change
            .append_handler_register(inspector_handle_register)
            .with_block_env(block_env.clone())
            .with_tx_env(tx_env)
            .build();
        evm.transact_commit().unwrap();
        (db, _) = evm.into_db_and_env_with_handler_cfg();
        progress_bar.inc(tx_outcome.interpreter_result.unwrap().gas.spent());
    }

    (block_env, tx_vec, db.init_db)
}

#[derive(Serialize, Deserialize)]
pub struct RecorderDB<ExtDB: DatabaseRef> {
    pub init_db: CacheDB<VoidDB>,
    pub running_db: CacheDB<VoidDB>,
    pub ext_db: ExtDB,
    pub path: String,
}

impl<ExtDB: DatabaseRef> DatabaseCommit for RecorderDB<ExtDB> {
    #[doc = " Commit changes to the database."]
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        self.running_db.commit(changes.clone());
    }
}

impl<ExtDB: DatabaseRef> RecorderDB<ExtDB> {
    pub fn new(ext_db: ExtDB, path: String) -> Self {
        let init_db = Self::load(&path).unwrap_or_else(CacheDB::<VoidDB>::default);
        Self {
            init_db,
            running_db: CacheDB::<VoidDB>::default(),
            ext_db,
            path,
        }
    }

    fn save(&self) {
        let buff = bincode::serialize(&self.init_db).expect("bincode::serialize failed");
        std::fs::write(self.path.clone(), buff).expect("write failed");
    }

    fn load(path: &String) -> Option<CacheDB<VoidDB>> {
        if let Ok(buff) = std::fs::read(path) {
            bincode::deserialize(&buff).ok()
        } else {
            None
        }
    }
}

impl<ExtDB: DatabaseRef> Database for RecorderDB<ExtDB> {
    #[doc = " The database error type."]
    type Error = ExtDB::Error;

    #[doc = " Get basic account information."]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // check if data is present in the init db or running db
        if let Ok(result) = self
            .running_db
            .basic_ref(address)
            .or_else(|_| self.init_db.basic_ref(address))
        {
            return Ok(result);
        }

        // otherwise download the data from the internet
        let result = self.ext_db.basic_ref(address);
        if let Ok(Some(info)) = &result {
            let acc = DbAccount {
                info: info.clone(),
                ..Default::default()
            };
            self.running_db.accounts.insert(address, acc.clone());
            self.init_db.accounts.insert(address, acc);
            self.save();
        }

        result
    }

    #[doc = " Get account code by its hash."]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        // check if data is present in the init db or running db
        if let Ok(result) = self
            .running_db
            .code_by_hash_ref(code_hash)
            .or_else(|_| self.init_db.code_by_hash_ref(code_hash))
        {
            return Ok(result);
        }

        // otherwise download the data from the internet
        let result = self.ext_db.code_by_hash_ref(code_hash);
        if let Ok(code) = &result {
            self.running_db.contracts.insert(code_hash, code.clone());
            self.init_db.contracts.insert(code_hash, code.clone());
            self.save();
        }
        result
    }

    #[doc = " Get storage value of address at index."]
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        // check if data is present in the init db or running db
        if let Ok(result) = self
            .running_db
            .storage_ref(address, index)
            .or_else(|_| self.init_db.storage_ref(address, index))
        {
            return Ok(result);
        }

        // otherwise download the data from the internet
        let result = self.ext_db.storage_ref(address, index);
        if let Ok(value) = &result {
            self.running_db
                .accounts
                .entry(address)
                .or_default()
                .storage
                .insert(index, *value);
            self.init_db
                .accounts
                .entry(address)
                .or_default()
                .storage
                .insert(index, *value);
            self.save();
        }
        result
    }

    #[doc = " Get block hash by block number."]
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        // check if data is present in the init db or running db
        if let Ok(result) = self
            .running_db
            .block_hash_ref(number)
            .or_else(|_| self.init_db.block_hash_ref(number))
        {
            return Ok(result);
        }

        // otherwise download the data from the internet
        let result = self.ext_db.block_hash_ref(number);
        if let Ok(hash) = &result {
            self.running_db.block_hashes.insert(number, *hash);
            self.init_db.block_hashes.insert(number, *hash);
            self.save();
        }
        result
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct VoidDB;

pub enum VoidError {
    AddressNotInDB(Address),
    ContractNotInDB(B256),
    StorageNotInDB(Address, U256),
    BlockHashNotInDB(U256),
}

impl DatabaseRef for VoidDB {
    type Error = VoidError;

    #[inline]
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Err(VoidError::AddressNotInDB(address))
    }

    #[inline]
    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Err(VoidError::ContractNotInDB(code_hash))
    }

    #[inline]
    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Err(VoidError::StorageNotInDB(address, index))
    }

    #[inline]
    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        Err(VoidError::BlockHashNotInDB(number))
    }
}
