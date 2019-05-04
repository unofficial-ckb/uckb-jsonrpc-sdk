// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use futures::{future, Future};
use tokio::runtime::Runtime;

use jsonrpc_sdk_client::r#async::Client;
use jsonrpc_sdk_prelude::{Error, Result};

use ckb_jsonrpc_interfaces::{core, types, Ckb, H256};

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
            .map(::std::convert::Into::into)
            .and_then(|r: String| {
                r.parse()
                    .map_err(|_| Error::custom("parse block number failed"))
            })
    }

    pub fn tip_header(&self) -> impl Future<Item = types::Header, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_tip_header(), Default::default())
            .map(::std::convert::Into::into)
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
                .map(::std::convert::Into::into)
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
                .map(::std::convert::Into::into)
                .and_then(|r: Option<types::Block>| {
                    r.ok_or_else(|| Error::custom("fetch block failed"))
                })
        })
    }

    pub fn block_by_hash(&self, hash: H256) -> impl Future<Item = types::Block, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_block(hash), Default::default())
            .map(::std::convert::Into::into)
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
                .map(::std::convert::Into::into)
        })
    }

    pub fn live_cell(
        &self,
        out_point: types::OutPoint,
    ) -> impl Future<Item = types::CellWithStatus, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_live_cell(out_point), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn total_capacity(
        &self,
        lock: &core::script::Script,
    ) -> impl Future<Item = u64, Error = Error> {
        self.cells_by_lock_hash(lock, None, None).and_then(|u| {
            u.into_iter()
                .map(|c| c.capacity.parse::<u64>())
                .collect::<::std::result::Result<Vec<_>, ::std::num::ParseIntError>>()
                .map_err(|_| Error::custom("parse capacity failed"))
                .and_then(|caps| {
                    caps.into_iter()
                        .try_fold(0u64, u64::checked_add)
                        .ok_or_else(|| Error::custom("sum capacity overflow"))
                })
        })
    }

    pub fn send(&self, tx: types::Transaction) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::send_transaction(tx), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn transaction(
        &self,
        hash: H256,
    ) -> impl Future<Item = types::TransactionWithStatus, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_transaction(hash), Default::default())
            .map(::std::convert::Into::into)
            .and_then(|r: Option<types::TransactionWithStatus>| {
                r.ok_or_else(|| Error::custom("fetch transaction with status failed"))
            })
    }

    pub fn trace(&self, tx: types::Transaction) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::trace_transaction(tx), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn transaction_trace(
        &self,
        hash: H256,
    ) -> impl Future<Item = Vec<types::TxTrace>, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_transaction_trace(hash), Default::default())
            .map(::std::convert::Into::into)
            .and_then(|r: Option<Vec<types::TxTrace>>| {
                r.ok_or_else(|| Error::custom("fetch transaction trace failed"))
            })
    }

    pub fn tx_pool_info(&self) -> impl Future<Item = types::TxPoolInfo, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::tx_pool_info(), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn local_node_info(&self) -> impl Future<Item = types::Node, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::local_node_info(), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn get_peers(&self) -> impl Future<Item = Vec<types::Node>, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_peers(), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn add_node(
        &self,
        peer_id: String,
        address: String,
    ) -> impl Future<Item = (), Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::add_node(peer_id, address), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn enqueue(&self, tx: types::Transaction) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url)
            .send(Ckb::enqueue_test_transaction(tx), Default::default())
            .map(::std::convert::Into::into)
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
