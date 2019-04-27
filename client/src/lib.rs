// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::convert::TryInto;
use std::sync::Arc;

use futures::{future, Future};
use tokio::runtime::Runtime;

use jsonrpc_sdk_client::Client;
use jsonrpc_sdk_prelude::{Error, Result};

use ckb_jsonrpc_interfaces::{core, types, Ckb, OccupiedCapacity, H256};

pub struct CkbClient {
    cli: Arc<Client>,
    url: Arc<String>,
    rt: Runtime,
}

impl CkbClient {
    pub fn new(url: &str) -> Self {
        Self {
            cli: Arc::new(Client::new()),
            url: Arc::new(url.to_owned()),
            rt: Runtime::new().unwrap(),
        }
    }

    /*
     * Basic
     */

    pub fn tip_block_number(&self) -> impl Future<Item = core::BlockNumber, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_tip_block_number(), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: String| {
                r.parse()
                    .map_err(|_| Error::custom("parse block number failed"))
            })
    }

    pub fn tip_header(&self) -> impl Future<Item = types::Header, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_tip_header(), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn block_hash(
        &self,
        height: Option<core::BlockNumber>,
    ) -> impl Future<Item = H256, Error = Error> {
        let cli = Arc::clone(&self.cli);
        let url = Arc::clone(&self.url);
        let fut = self.tip_block_number();
        if let Some(h) = height {
            future::ok(h)
        } else {
            future::err(Error::none())
        }
        .or_else(|_| fut)
        .and_then(move |h| {
            cli.post(&*url)
                .send(Ckb::get_block_hash(h.to_string()), Default::default())
                .map(std::convert::Into::into)
                .and_then(|r: Option<H256>| {
                    r.ok_or_else(|| Error::custom("fetch block hash failed"))
                })
        })
    }

    pub fn block_by_number(
        &self,
        height: Option<core::BlockNumber>,
    ) -> impl Future<Item = types::Block, Error = Error> {
        let cli = Arc::clone(&self.cli);
        let url = Arc::clone(&self.url);
        self.block_hash(height).and_then(move |r| {
            cli.post(&*url)
                .send(Ckb::get_block(r), Default::default())
                .map(std::convert::Into::into)
                .and_then(|r: Option<types::Block>| {
                    r.ok_or_else(|| Error::custom("fetch block failed"))
                })
        })
    }

    pub fn block_by_hash(&self, hash: H256) -> impl Future<Item = types::Block, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_block(hash), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: Option<types::Block>| {
                r.ok_or_else(|| Error::custom("fetch block failed"))
            })
    }

    pub fn genesis_block(&self) -> impl Future<Item = types::Block, Error = Error> {
        self.block_by_number(Some(0))
    }

    pub fn last_block(&self) -> impl Future<Item = types::Block, Error = Error> {
        self.block_by_number(None)
    }

    pub fn cells_by_lock_hash(
        &self,
        lock: &core::script::Script,
        from: Option<core::BlockNumber>,
        to: Option<core::BlockNumber>,
    ) -> impl Future<Item = Vec<types::CellOutputWithOutPoint>, Error = Error> {
        let lock_hash = lock.hash();
        let cli = Arc::clone(&self.cli);
        let url = Arc::clone(&self.url);
        let from = from.unwrap_or(0);
        let fut = self.tip_block_number();
        if let Some(h) = to {
            future::ok(h)
        } else {
            future::err(Error::none())
        }
        .or_else(|_| fut)
        .and_then(move |to| {
            cli.post(&*url)
                .send(
                    Ckb::get_cells_by_lock_hash(lock_hash, from.to_string(), to.to_string()),
                    Default::default(),
                )
                .map(std::convert::Into::into)
        })
    }

    pub fn live_cell(
        &self,
        out_point: types::OutPoint,
    ) -> impl Future<Item = types::CellWithStatus, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_live_cell(out_point), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn total_capacity(
        &self,
        lock: &core::script::Script,
    ) -> impl Future<Item = core::Capacity, Error = Error> {
        self.cells_by_lock_hash(lock, None, None)
            .map(|u| u.into_iter().map(|c| c.capacity).sum())
    }

    pub fn send(&self, tx: types::Transaction) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::send_transaction(tx), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn pool_transaction(
        &self,
        hash: H256,
    ) -> impl Future<Item = types::Transaction, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_pool_transaction(hash), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: Option<types::Transaction>| {
                r.ok_or_else(|| Error::custom("fetch pool transaction failed"))
            })
    }

    pub fn transaction(&self, hash: H256) -> impl Future<Item = types::Transaction, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_transaction(hash), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: Option<types::Transaction>| {
                r.ok_or_else(|| Error::custom("fetch transaction failed"))
            })
    }

    pub fn trace(&self, tx: types::Transaction) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::trace_transaction(tx), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn transaction_trace(
        &self,
        hash: H256,
    ) -> impl Future<Item = Vec<types::TxTrace>, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_transaction_trace(hash), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: Option<Vec<types::TxTrace>>| {
                r.ok_or_else(|| Error::custom("fetch transaction trace failed"))
            })
    }

    pub fn local_node_info(&self) -> impl Future<Item = types::Node, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::local_node_info(), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn get_peers(&self) -> impl Future<Item = Vec<types::Node>, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_peers(), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn add_node(
        &self,
        peer_id: String,
        address: String,
    ) -> impl Future<Item = (), Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::add_node(peer_id, address), Default::default())
            .map(std::convert::Into::into)
    }

    /*
     * Combine
     */

    pub fn gather(
        &self,
        lock_in: &core::script::Script,
        lock_out: &core::script::Script,
        from: Option<core::BlockNumber>,
        to: Option<core::BlockNumber>,
    ) -> impl Future<Item = types::Transaction, Error = Error> {
        let lock_out = lock_out.clone();
        self.cells_by_lock_hash(lock_in, from, to).map(
            move |cells: Vec<types::CellOutputWithOutPoint>| {
                let capacity = cells.iter().map(|c| c.capacity).sum();
                let inputs = cells
                    .into_iter()
                    .map(|c| {
                        core::transaction::CellInput {
                            previous_output: c.out_point.try_into().unwrap(),
                            args: vec![],
                            valid_since: 0,
                        }
                        .into()
                    })
                    .collect();
                let output =
                    core::transaction::CellOutput::new(capacity, Vec::new(), lock_out, None);
                types::Transaction {
                    version: 0,
                    deps: vec![],
                    inputs,
                    outputs: vec![output.into()],
                    witnesses: vec![],
                    hash: Default::default(),
                }
            },
        )
    }

    pub fn disperse(
        &self,
        lock_in: &core::script::Script,
        lock_out: &core::script::Script,
        from: Option<core::BlockNumber>,
        to: Option<core::BlockNumber>,
        max_count: usize,
    ) -> impl Future<Item = types::Transaction, Error = Error> {
        let lock_out = lock_out.clone();
        self.cells_by_lock_hash(lock_in, from, to)
            .and_then(|cells| {
                if cells.is_empty() {
                    Err(Error::custom("input is empty"))
                } else {
                    Ok(cells)
                }
            })
            .map(move |cells: Vec<types::CellOutputWithOutPoint>| {
                let mut capacity: u64 = cells.iter().map(|c| c.capacity).sum();
                let inputs = cells
                    .into_iter()
                    .map(|c| {
                        core::transaction::CellInput {
                            previous_output: c.out_point.try_into().unwrap(),
                            args: vec![],
                            valid_since: 0,
                        }
                        .into()
                    })
                    .collect();
                let mut outputs = Vec::new();
                while capacity > 0 && outputs.len() < max_count {
                    let mut output =
                        core::transaction::CellOutput::new(0, Vec::new(), lock_out.clone(), None);
                    output.capacity = output.occupied_capacity() as u64;
                    if capacity < output.capacity {
                        break;
                    }
                    capacity -= output.capacity;
                    outputs.push(output);
                }
                if capacity > 0 {
                    outputs[0].capacity += capacity;
                }
                types::Transaction {
                    version: 0,
                    deps: vec![],
                    inputs,
                    outputs: outputs.into_iter().map(Into::into).collect(),
                    witnesses: vec![],
                    hash: Default::default(),
                }
            })
    }

    /*
     * Utilities
     */

    pub fn block_on<F, R>(&mut self, future: F) -> Result<R>
    where
        F: Send + 'static + Future<Item = R, Error = Error>,
        R: Send + 'static,
    {
        self.rt.block_on(future)
    }

    pub fn until_ok<F, R>(&mut self, future: F, limit_times: u64, interval_secs: u64) -> Result<R>
    where
        F: Send + 'static + Future<Item = R, Error = Error>,
        R: Send + 'static,
        F: Clone,
    {
        let mut cnt = 0;
        let wait_secs = ::std::time::Duration::from_secs(interval_secs);
        while cnt < limit_times {
            if let Ok(r) = self.rt.block_on(future.clone()) {
                return Ok(r);
            } else {
                cnt += 1;
                ::std::thread::sleep(wait_secs);
                continue;
            }
        }
        Err(Error::custom("waiting too long"))
    }
}
