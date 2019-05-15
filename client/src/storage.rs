// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    fs,
    path::Path,
    result::Result as StdResult,
    sync::{Arc, RwLock},
};

use bincode;
use property::Property;
use rkv::{Manager, Reader, Rkv, SingleStore, StoreError, StoreOptions, Value, Writer};

use ckb_jsonrpc_interfaces::{core, types, H256};

use crate::error::{Error, Result};
use crate::serde::{
    deserialize, deserialize_block, deserialize_transaction, deserialize_uncle_block,
    serialize_block_with_index, BlockIndex,
};

const MAX_READERS: u32 = 64;
const MAX_DBS: u32 = 16;
const MAP_SIZE: usize = 1024 * 1024 * 1024;

#[derive(Debug, Default)]
struct Index(u64, usize);

struct CellIndex(H256, usize);

pub type RkvResult<T> = StdResult<T, StoreError>;

pub trait StorageWriter {
    fn insert_block(&self, block: &types::BlockView) -> Result<()>;
    fn insert<K: AsRef<[u8]>>(&self, key: K, value: &[u8]) -> Result<()>;
}

pub trait StorageReader {
    fn select_block_hash(&self, block_number: u64) -> Result<Option<H256>>;
    fn select_block_number(&self, block_hash: &H256) -> Result<Option<core::BlockNumber>>;
    fn select_block_by_number(&self, block_number: u64) -> Result<Option<types::BlockView>>;
    fn select_block_by_hash(&self, block_hash: &H256) -> Result<Option<types::BlockView>>;
    fn select_uncle_block(&self, uncle_hash: &H256) -> Result<Option<types::UncleBlockView>>;
    fn select_transaction(&self, tx_hash: &H256) -> Result<Option<types::TransactionView>>;
    fn select_cell_output(
        &self,
        tx_hash: &H256,
        cell_index: u32,
    ) -> Result<Option<types::CellOutput>>;
    fn select_cell_status(&self, tx_hash: &H256, cell_index: u32) -> Result<Option<()>>;
    fn select_max_number(&self) -> Result<Option<u64>>;
    fn select<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Vec<u8>>>;
}

#[derive(Property)]
#[property(get(crate), set(disable), get_mut(disable))]
pub struct Storage {
    codecfg: bincode::Config,

    #[property(get(disable))]
    env: Arc<RwLock<Rkv>>,

    // Data: Block Number -> Block Index
    indexes: SingleStore,
    // Data: Block Number -> Block Data
    blocks: SingleStore,

    // Index: Block Number -> Block Hash
    hashes: SingleStore,
    // Index: Block Hash -> Block Number
    numbers: SingleStore,

    // Index: Uncle Block Hash -> Block Number, Index Number
    uncles: SingleStore,
    // Index: Transaction Hash -> Block Number, Index Number
    txs: SingleStore,

    // Alive Cells
    cells: SingleStore,

    // Status
    status: SingleStore,

    // Custom
    custom: SingleStore,
}

impl Index {
    fn new(number: u64, index: usize) -> Self {
        Index(number, index)
    }

    fn into_bytes(self) -> [u8; 16] {
        let Index(number, index) = self;
        let mut bytes = [0u8; 16];
        (&mut bytes[0..8]).copy_from_slice(&number.to_le_bytes());
        (&mut bytes[8..16]).copy_from_slice(&(index as u64).to_le_bytes());
        bytes
    }

    fn from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&slice[0..8]);
        let number = u64::from_le_bytes(bytes);
        bytes.copy_from_slice(&slice[8..16]);
        let index = u64::from_le_bytes(bytes) as usize;
        Index(number, index)
    }
}

impl CellIndex {
    fn new(tx_hash: H256, index: usize) -> Self {
        CellIndex(tx_hash, index)
    }

    fn into_bytes(self) -> [u8; 36] {
        let CellIndex(tx_hash, index) = self;
        let mut bytes = [0u8; 36];
        (&mut bytes[0..32]).copy_from_slice(&tx_hash.as_bytes());
        (&mut bytes[32..36]).copy_from_slice(&(index as u32).to_le_bytes());
        bytes
    }
}

fn build_env(path: &Path) -> RkvResult<Rkv> {
    let mut builder = Rkv::environment_builder();
    builder.set_max_readers(MAX_READERS);
    builder.set_max_dbs(MAX_DBS);
    builder.set_map_size(MAP_SIZE);
    Rkv::from_env(path, builder)
}

macro_rules! return_ok_if_none {
    ($opt:ident) => {
        if let Some(v) = $opt {
            v
        } else {
            return Ok(None);
        }
    };
}

impl Storage {
    pub fn initial(path_str: &str) -> RkvResult<Self> {
        let path = Path::new(path_str);
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        let codecfg = bincode::config();
        let env = Manager::singleton()
            .write()
            .expect("failed to unlock singleton for write")
            .get_or_create(path, build_env)?;
        let (indexes, blocks, hashes, numbers, uncles, txs, cells, status, custom) = {
            let env = env.read().expect("failed to read environment");
            let indexes = env.open_single("indexes", StoreOptions::create())?;
            let blocks = env.open_single("blocks", StoreOptions::create())?;
            let hashes = env.open_single("hashes", StoreOptions::create())?;
            let numbers = env.open_single("numbers", StoreOptions::create())?;
            let uncles = env.open_single("uncles", StoreOptions::create())?;
            let txs = env.open_single("transactions", StoreOptions::create())?;
            let cells = env.open_single("cells", StoreOptions::create())?;
            let status = env.open_single("status", StoreOptions::create())?;
            let custom = env.open_single("custom", StoreOptions::create())?;
            (
                indexes, blocks, hashes, numbers, uncles, txs, cells, status, custom,
            )
        };
        Ok(Self {
            codecfg,
            env,
            indexes,
            blocks,
            hashes,
            numbers,
            uncles,
            txs,
            cells,
            status,
            custom,
        })
    }

    pub(crate) fn write<F, T>(&self, callback: F) -> Result<T>
    where
        F: FnOnce(Writer) -> Result<T>,
    {
        let handler = self.env.read().expect("failed to get a handler");
        let writer = handler.write()?;
        callback(writer)
    }

    pub(crate) fn read<F, T>(&self, callback: F) -> Result<T>
    where
        F: FnOnce(&Reader) -> Result<T>,
    {
        let handler = self.env.read().expect("failed to get a handler");
        let reader = handler.read()?;
        callback(&reader)
    }
}

impl StorageWriter for Storage {
    fn insert_block(&self, block: &types::BlockView) -> Result<()> {
        let config = self.get_codecfg();
        let (block_bytes, index_bytes) = serialize_block_with_index(config, block)?;
        let number = block.header.inner.number.0;
        let number_bytes = number.to_le_bytes();
        let block_hash = block.header.hash.as_bytes();
        let uncles = block
            .uncles
            .iter()
            .map(|uncle| uncle.header.hash.as_bytes())
            .enumerate()
            .map(|(index, hash)| (hash, Index::new(number, index).into_bytes()))
            .collect::<Vec<(&[u8], [u8; 16])>>();
        let txs = block
            .transactions
            .iter()
            .map(|tx| {
                let tx_hash = &tx.hash;
                let inputs = tx
                    .inner
                    .inputs
                    .iter()
                    .filter_map(|input| {
                        let cell = &input.previous_output.cell;
                        cell.to_owned().map(|cell| {
                            CellIndex::new(cell.tx_hash.to_owned(), cell.index.0 as usize)
                        })
                    })
                    .collect::<Vec<_>>();
                let outputs = tx
                    .inner
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(i, _)| CellIndex::new(tx_hash.to_owned(), i))
                    .collect::<Vec<_>>();
                (tx_hash, inputs, outputs)
            })
            .enumerate()
            .map(|(index, (hash, inputs, outputs))| {
                (
                    hash,
                    Index::new(number, index).into_bytes(),
                    inputs,
                    outputs,
                )
            })
            .collect::<Vec<(&H256, [u8; 16], Vec<CellIndex>, Vec<CellIndex>)>>();
        self.write(|mut writer| {
            self.indexes
                .put(&mut writer, &number_bytes, &Value::Blob(&index_bytes))?;
            self.blocks
                .put(&mut writer, &number_bytes, &Value::Blob(&block_bytes))?;
            self.hashes
                .put(&mut writer, &number_bytes, &Value::Blob(&block_hash))?;
            self.numbers
                .put(&mut writer, &block_hash, &Value::U64(number))?;
            for (hash, index) in uncles.into_iter() {
                self.uncles.put(&mut writer, &hash, &Value::Blob(&index))?;
            }
            for (hash, index, inputs, outputs) in txs.into_iter() {
                self.txs
                    .put(&mut writer, &hash.as_bytes(), &Value::Blob(&index))?;
                for input in inputs.into_iter() {
                    self.cells.delete(&mut writer, &input.into_bytes()[..])?;
                }
                for output in outputs.into_iter() {
                    self.cells
                        .put(&mut writer, &output.into_bytes()[..], &Value::Bool(true))?;
                }
            }
            self.status
                .put(&mut writer, "max-number", &Value::U64(number))?;
            writer.commit()?;
            Ok(())
        })
    }

    fn insert<K: AsRef<[u8]>>(&self, key: K, value: &[u8]) -> Result<()> {
        self.write(|mut writer| {
            self.custom.put(&mut writer, key, &Value::Blob(value))?;
            writer.commit()?;
            Ok(())
        })
    }
}

impl StorageReader for Storage {
    fn select_block_hash(&self, block_number: u64) -> Result<Option<H256>> {
        let number_bytes = block_number.to_le_bytes();
        self.read(|reader| {
            let hash_opt = self.hashes.get(reader, &number_bytes)?;
            let hash_blob = return_ok_if_none!(hash_opt);
            let hash_bytes = unpack_blob(hash_blob)?;
            let hash = unpack_hash(hash_bytes)?;
            Ok(Some(hash))
        })
    }

    fn select_block_number(&self, block_hash: &H256) -> Result<Option<core::BlockNumber>> {
        let hash_bytes = block_hash.as_bytes();
        self.read(|reader| {
            let number_opt = self.numbers.get(reader, &hash_bytes)?;
            let number_value = return_ok_if_none!(number_opt);
            let number = unpack_u64(number_value)?;
            Ok(Some(number))
        })
    }

    fn select_block_by_number(&self, block_number: u64) -> Result<Option<types::BlockView>> {
        let config = self.get_codecfg();
        let number_bytes = block_number.to_le_bytes();
        self.read(|reader| {
            let block_index_opt = self.indexes.get(reader, &number_bytes)?;
            let block_index_blob = return_ok_if_none!(block_index_opt);
            let block_index_bytes = unpack_blob(block_index_blob)?;

            let block_opt = self.blocks.get(reader, &number_bytes)?;
            let block_blob = return_ok_if_none!(block_opt);
            let block_bytes = unpack_blob(block_blob)?;

            let block_index: BlockIndex = config.deserialize(block_index_bytes)?;
            let block = deserialize_block(config, block_bytes, &block_index)?;
            Ok(Some(block))
        })
    }

    fn select_block_by_hash(&self, block_hash: &H256) -> Result<Option<types::BlockView>> {
        let config = self.get_codecfg();
        let hash_bytes = block_hash.as_bytes();
        self.read(|reader| {
            let number_opt = self.numbers.get(reader, &hash_bytes)?;
            let number_value = return_ok_if_none!(number_opt);
            let number = unpack_u64(number_value)?;

            let number_bytes = number.to_le_bytes();

            let block_index_opt = self.indexes.get(reader, &number_bytes)?;
            let block_index_blob = return_ok_if_none!(block_index_opt);
            let block_index_bytes = unpack_blob(block_index_blob)?;

            let block_opt = self.blocks.get(reader, &number_bytes)?;
            let block_blob = return_ok_if_none!(block_opt);
            let block_bytes = unpack_blob(block_blob)?;

            let block_index: BlockIndex = config.deserialize(block_index_bytes)?;
            let block = deserialize_block(config, block_bytes, &block_index)?;
            Ok(Some(block))
        })
    }

    fn select_uncle_block(&self, uncle_hash: &H256) -> Result<Option<types::UncleBlockView>> {
        let config = self.get_codecfg();
        let hash_bytes = uncle_hash.as_bytes();
        self.read(|reader| {
            let index_opt = self.uncles.get(reader, &hash_bytes)?;
            let index_blob = return_ok_if_none!(index_opt);
            let index_bytes = unpack_blob(index_blob)?;
            let Index(number, index) = Index::from_slice(index_bytes);

            let number_bytes = number.to_le_bytes();

            let block_index_opt = self.indexes.get(reader, &number_bytes)?;
            let block_index_blob = return_ok_if_none!(block_index_opt);
            let block_index_bytes = unpack_blob(block_index_blob)?;

            let block_opt = self.blocks.get(reader, &number_bytes)?;
            let block_blob = return_ok_if_none!(block_opt);
            let block_bytes = unpack_blob(block_blob)?;

            let block_index: BlockIndex = config.deserialize(block_index_bytes)?;
            let uncle = deserialize_uncle_block(config, block_bytes, &block_index.uncles[index])?;
            Ok(Some(uncle))
        })
    }

    fn select_transaction(&self, tx_hash: &H256) -> Result<Option<types::TransactionView>> {
        let config = self.get_codecfg();
        let hash_bytes = tx_hash.as_bytes();
        self.read(|reader| {
            let index_opt = self.txs.get(reader, &hash_bytes)?;
            let index_blob = return_ok_if_none!(index_opt);
            let index_bytes = unpack_blob(index_blob)?;
            let Index(number, index) = Index::from_slice(index_bytes);

            let number_bytes = number.to_le_bytes();

            let block_index_opt = self.indexes.get(reader, &number_bytes)?;
            let block_index_blob = return_ok_if_none!(block_index_opt);
            let block_index_bytes = unpack_blob(block_index_blob)?;

            let block_opt = self.blocks.get(reader, &number_bytes)?;
            let block_blob = return_ok_if_none!(block_opt);
            let block_bytes = unpack_blob(block_blob)?;

            let block_index: BlockIndex = config.deserialize(block_index_bytes)?;
            let tx =
                deserialize_transaction(config, block_bytes, &block_index.transactions[index])?;
            Ok(Some(tx))
        })
    }

    fn select_cell_output(
        &self,
        tx_hash: &H256,
        cell_index: u32,
    ) -> Result<Option<types::CellOutput>> {
        let config = self.get_codecfg();
        let hash_bytes = tx_hash.as_bytes();
        self.read(|reader| {
            let index_opt = self.txs.get(reader, &hash_bytes)?;
            let index_blob = return_ok_if_none!(index_opt);
            let index_bytes = unpack_blob(index_blob)?;
            let Index(number, index) = Index::from_slice(index_bytes);

            let number_bytes = number.to_le_bytes();

            let block_index_opt = self.indexes.get(reader, &number_bytes)?;
            let block_index_blob = return_ok_if_none!(block_index_opt);
            let block_index_bytes = unpack_blob(block_index_blob)?;

            let block_opt = self.blocks.get(reader, &number_bytes)?;
            let block_blob = return_ok_if_none!(block_opt);
            let block_bytes = unpack_blob(block_blob)?;

            let block_index: BlockIndex = config.deserialize(block_index_bytes)?;
            let position = &block_index.transactions[index].outputs[cell_index as usize];
            let cell_output = deserialize(config, block_bytes, position)?;
            Ok(Some(cell_output))
        })
    }

    fn select_cell_status(&self, tx_hash: &H256, cell_index: u32) -> Result<Option<()>> {
        let index = CellIndex::new(tx_hash.to_owned(), cell_index as usize);
        self.read(|reader| {
            let cell_opt = self.cells.get(reader, &index.into_bytes()[..])?;
            let _ = return_ok_if_none!(cell_opt);
            Ok(Some(()))
        })
    }

    fn select_max_number(&self) -> Result<Option<u64>> {
        self.read(|reader| {
            let number_opt = self.status.get(reader, "max-number")?;
            let number_value = return_ok_if_none!(number_opt);
            let number = unpack_u64(number_value)?;
            Ok(Some(number))
        })
    }

    fn select<T: AsRef<[u8]>>(&self, key: T) -> Result<Option<Vec<u8>>> {
        self.read(|reader| {
            let data_opt = self.custom.get(reader, key)?;
            let data_blob = return_ok_if_none!(data_opt);
            let data_bytes = unpack_blob(data_blob)?;
            Ok(Some(data_bytes.to_vec()))
        })
    }
}

fn unpack_u64(value: Value) -> Result<u64> {
    if let Value::U64(number) = value {
        Ok(number)
    } else {
        Err(Error::corruption("value should be number"))
    }
}

fn unpack_blob(value: Value) -> Result<&[u8]> {
    if let Value::Blob(bytes) = value {
        Ok(bytes)
    } else {
        Err(Error::corruption("value should be blob"))
    }
}

fn unpack_hash(bytes: &[u8]) -> Result<H256> {
    H256::from_slice(bytes).map_err(|_| Error::corruption("value should be hash"))
}
