// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::Write;

use bincode;
use serde;
use serde_derive::{Deserialize, Serialize};

use ckb_jsonrpc_interfaces::types;

pub(crate) type Size = usize;
pub(crate) type Bytes = Vec<u8>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Position {
    pub(crate) offset: Size,
    pub(crate) length: Size,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BlockIndex {
    pub(crate) hash: Position,
    pub(crate) header: Position,
    pub(crate) uncles: Vec<UncleBlockIndex>,
    pub(crate) transactions: Vec<TransactionIndex>,
    pub(crate) proposals: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UncleBlockIndex {
    pub(crate) hash: Position,
    pub(crate) header: Position,
    pub(crate) proposals: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TransactionIndex {
    pub(crate) hash: Position,
    pub(crate) version: Position,
    pub(crate) deps: Vec<Position>,
    pub(crate) inputs: Vec<Position>,
    pub(crate) outputs: Vec<Position>,
    pub(crate) witnesses: Vec<Position>,
}

#[derive(Debug)]
pub(crate) struct BlockSize {
    pub(crate) hash: Size,
    pub(crate) header: Size,
    pub(crate) uncles: Vec<UncleBlockSize>,
    pub(crate) transactions: Vec<TransactionSize>,
    pub(crate) proposals: Vec<Size>,
}

#[derive(Debug)]
pub(crate) struct UncleBlockSize {
    pub(crate) hash: Size,
    pub(crate) header: Size,
    pub(crate) proposals: Vec<Size>,
}

#[derive(Debug)]
pub(crate) struct TransactionSize {
    pub(crate) hash: Size,
    pub(crate) version: Size,
    pub(crate) deps: Vec<Size>,
    pub(crate) inputs: Vec<Size>,
    pub(crate) outputs: Vec<Size>,
    pub(crate) witnesses: Vec<Size>,
}

impl Position {
    fn new(offset: Size, length: Size) -> Self {
        Self { offset, length }
    }
}

fn serialized_size<'a, T>(config: &bincode::Config, data: &'a T) -> bincode::Result<Size>
where
    T: 'a + serde::ser::Serialize,
{
    config.serialized_size(data).map(|s| s as Size)
}

fn serialized_sizes<'a, T>(
    config: &bincode::Config,
    iter: impl Iterator<Item = &'a T>,
) -> bincode::Result<Vec<Size>>
where
    T: 'a + serde::ser::Serialize,
{
    iter.map(|ref data| Ok(config.serialized_size(data)? as Size))
        .collect::<bincode::Result<Vec<Size>>>()
}

fn serialized_block_size(
    config: &bincode::Config,
    block: &types::BlockView,
) -> bincode::Result<BlockSize> {
    let hash = serialized_size(config, &block.header.hash)?;
    let header = serialized_size(config, &block.header.inner)?;
    let uncles = block
        .uncles
        .iter()
        .map(|ref data| serialized_uncle_block_size(config, data))
        .collect::<bincode::Result<Vec<UncleBlockSize>>>()?;
    let transactions = block
        .transactions
        .iter()
        .map(|ref data| serialized_transaction_size(config, data))
        .collect::<bincode::Result<Vec<TransactionSize>>>()?;
    let proposals = serialized_sizes(config, block.proposals.iter())?;
    Ok(BlockSize {
        hash,
        header,
        uncles,
        transactions,
        proposals,
    })
}

fn serialized_uncle_block_size(
    config: &bincode::Config,
    uncle: &types::UncleBlockView,
) -> bincode::Result<UncleBlockSize> {
    let hash = serialized_size(config, &uncle.header.hash)?;
    let header = serialized_size(config, &uncle.header.inner)?;
    let proposals = serialized_sizes(config, uncle.proposals.iter())?;
    Ok(UncleBlockSize {
        hash,
        header,
        proposals,
    })
}

fn serialized_transaction_size(
    config: &bincode::Config,
    tx_view: &types::TransactionView,
) -> bincode::Result<TransactionSize> {
    let tx = &tx_view.inner;
    let hash = serialized_size(config, &tx_view.hash)?;
    let version = serialized_size(config, &tx.version)?;
    let deps = serialized_sizes(config, tx.deps.iter())?;
    let inputs = serialized_sizes(config, tx.inputs.iter())?;
    let outputs = serialized_sizes(config, tx.outputs.iter())?;
    let witnesses = serialized_sizes(config, tx.witnesses.iter())?;
    Ok(TransactionSize {
        hash,
        version,
        deps,
        inputs,
        outputs,
        witnesses,
    })
}

fn calculate_position(offset: Size, length: Size) -> (Size, Position) {
    let offset_new = offset + length;
    let position = Position::new(offset, length);
    (offset_new, position)
}

fn calculate_positions(offset: Size, lengths: Vec<Size>) -> (Size, Vec<Position>) {
    let positions = Vec::with_capacity(lengths.len());
    lengths
        .into_iter()
        .fold((offset, positions), |(offset, mut positions), length| {
            let (offset, position) = calculate_position(offset, length);
            positions.push(position);
            (offset, positions)
        })
}

fn calculate_block_index(block: BlockSize) -> (Size, BlockIndex) {
    let offset = 0;
    let (offset, hash) = calculate_position(offset, block.hash);
    let (offset, header) = calculate_position(offset, block.header);
    let (offset, uncles) = {
        let positions = Vec::with_capacity(block.uncles.len());
        block
            .uncles
            .into_iter()
            .fold((offset, positions), |(offset, mut positions), length| {
                let (offset, position) = calculate_uncle_block_index(offset, length);
                positions.push(position);
                (offset, positions)
            })
    };
    let (offset, transactions) = {
        let positions = Vec::with_capacity(block.transactions.len());
        block.transactions.into_iter().fold(
            (offset, positions),
            |(offset, mut positions), length| {
                let (offset, position) = calculate_transaction_index(offset, length);
                positions.push(position);
                (offset, positions)
            },
        )
    };
    let (offset, proposals) = calculate_positions(offset, block.proposals);
    let index = BlockIndex {
        hash,
        header,
        uncles,
        transactions,
        proposals,
    };
    (offset, index)
}

fn calculate_uncle_block_index(offset: Size, uncle: UncleBlockSize) -> (Size, UncleBlockIndex) {
    let (offset, hash) = calculate_position(offset, uncle.hash);
    let (offset, header) = calculate_position(offset, uncle.header);
    let (offset, proposals) = calculate_positions(offset, uncle.proposals);
    let index = UncleBlockIndex {
        hash,
        header,
        proposals,
    };
    (offset, index)
}

fn calculate_transaction_index(offset: Size, tx: TransactionSize) -> (Size, TransactionIndex) {
    let (offset, hash) = calculate_position(offset, tx.hash);
    let (offset, version) = calculate_position(offset, tx.version);
    let (offset, deps) = calculate_positions(offset, tx.deps);
    let (offset, inputs) = calculate_positions(offset, tx.inputs);
    let (offset, outputs) = calculate_positions(offset, tx.outputs);
    let (offset, witnesses) = calculate_positions(offset, tx.witnesses);
    let index = TransactionIndex {
        hash,
        version,
        deps,
        inputs,
        outputs,
        witnesses,
    };
    (offset, index)
}

fn serialize<W, T>(config: &bincode::Config, w: W, data: &T) -> bincode::Result<()>
where
    W: Write,
    T: ?Sized + serde::ser::Serialize + ::std::fmt::Debug,
{
    config.serialize_into(w, data)
}

fn serialize_many<'a, T>(
    config: &bincode::Config,
    mut bytes: Bytes,
    iter: impl Iterator<Item = &'a T>,
) -> bincode::Result<Bytes>
where
    T: 'a + ?Sized + serde::ser::Serialize + ::std::fmt::Debug,
{
    for data in iter {
        config.serialize_into(&mut bytes, data)?;
    }
    Ok(bytes)
}

pub(crate) fn serialize_block_with_index(
    config: &bincode::Config,
    block: &types::BlockView,
) -> bincode::Result<(Bytes, Bytes)> {
    let block_size = serialized_block_size(config, block)?;
    let (block_length, block_index) = calculate_block_index(block_size);
    let empty_bytes = Vec::with_capacity(block_length);
    let block_bytes = serialize_block(config, empty_bytes, block)?;
    assert_eq!(block_length, block_bytes.len());
    let index_bytes = config.serialize(&block_index)?;
    Ok((block_bytes, index_bytes))
}

fn serialize_block(
    config: &bincode::Config,
    mut bytes: Bytes,
    block: &types::BlockView,
) -> bincode::Result<Bytes> {
    serialize(config, &mut bytes, &block.header.hash)?;
    serialize(config, &mut bytes, &block.header.inner)?;
    for uncle in block.uncles.iter() {
        bytes = serialize_uncle_block(config, bytes, uncle)?;
    }
    for tx in block.transactions.iter() {
        bytes = serialize_transaction(config, bytes, tx)?;
    }
    bytes = serialize_many(config, bytes, block.proposals.iter())?;
    Ok(bytes)
}

fn serialize_uncle_block(
    config: &bincode::Config,
    mut bytes: Bytes,
    uncle: &types::UncleBlockView,
) -> bincode::Result<Bytes> {
    serialize(config, &mut bytes, &uncle.header.hash)?;
    serialize(config, &mut bytes, &uncle.header.inner)?;
    bytes = serialize_many(config, bytes, uncle.proposals.iter())?;
    Ok(bytes)
}

fn serialize_transaction(
    config: &bincode::Config,
    mut bytes: Bytes,
    tx_view: &types::TransactionView,
) -> bincode::Result<Bytes> {
    let tx = &tx_view.inner;
    serialize(config, &mut bytes, &tx_view.hash)?;
    serialize(config, &mut bytes, &tx.version)?;
    bytes = serialize_many(config, bytes, tx.deps.iter())?;
    bytes = serialize_many(config, bytes, tx.inputs.iter())?;
    bytes = serialize_many(config, bytes, tx.outputs.iter())?;
    bytes = serialize_many(config, bytes, tx.witnesses.iter())?;
    Ok(bytes)
}

pub(crate) fn deserialize<'a, T>(
    config: &bincode::Config,
    bytes: &'a [u8],
    position: &Position,
) -> bincode::Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    config.deserialize(&bytes[position.offset..(position.offset + position.length)])
}

fn deserialize_many<'a, T>(
    config: &bincode::Config,
    bytes: &'a [u8],
    positions: &[Position],
) -> bincode::Result<Vec<T>>
where
    T: serde::de::Deserialize<'a>,
{
    positions
        .iter()
        .map(|position| {
            config.deserialize(&bytes[position.offset..(position.offset + position.length)])
        })
        .collect()
}

pub(crate) fn deserialize_block(
    config: &bincode::Config,
    bytes: &[u8],
    index: &BlockIndex,
) -> bincode::Result<types::BlockView> {
    Ok(types::BlockView {
        header: types::HeaderView {
            hash: deserialize(config, bytes, &index.hash)?,
            inner: deserialize(config, bytes, &index.header)?,
        },
        uncles: index
            .uncles
            .iter()
            .map(|index| deserialize_uncle_block(config, bytes, index))
            .collect::<bincode::Result<Vec<_>>>()?,
        transactions: index
            .transactions
            .iter()
            .map(|index| deserialize_transaction(config, bytes, index))
            .collect::<bincode::Result<Vec<_>>>()?,
        proposals: deserialize_many(config, bytes, &index.proposals)?,
    })
}

pub(crate) fn deserialize_uncle_block(
    config: &bincode::Config,
    bytes: &[u8],
    index: &UncleBlockIndex,
) -> bincode::Result<types::UncleBlockView> {
    Ok(types::UncleBlockView {
        header: types::HeaderView {
            hash: deserialize(config, bytes, &index.hash)?,
            inner: deserialize(config, bytes, &index.header)?,
        },
        proposals: deserialize_many(config, bytes, &index.proposals)?,
    })
}

pub(crate) fn deserialize_transaction(
    config: &bincode::Config,
    bytes: &[u8],
    index: &TransactionIndex,
) -> bincode::Result<types::TransactionView> {
    Ok(types::TransactionView {
        hash: deserialize(config, bytes, &index.hash)?,
        inner: types::Transaction {
            version: deserialize(config, bytes, &index.version)?,
            deps: deserialize_many(config, bytes, &index.deps[..])?,
            inputs: deserialize_many(config, bytes, &index.inputs[..])?,
            outputs: deserialize_many(config, bytes, &index.outputs[..])?,
            witnesses: deserialize_many(config, bytes, &index.witnesses[..])?,
        },
    })
}
