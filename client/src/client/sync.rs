// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use jsonrpc_sdk_client::sync::Client;
use jsonrpc_sdk_prelude::{Error, Result};

use ckb_jsonrpc_interfaces::{core, types, Ckb, H256};

pub struct CkbClient {
    cli: Arc<Client>,
    url: Arc<String>,
}

impl CkbClient {
    pub fn new(url: url::Url) -> Self {
        Self {
            cli: Arc::new(Client::new()),
            url: Arc::new(url.into_string()),
        }
    }

    pub fn cli(&self) -> Arc<Client> {
        Arc::clone(&self.cli)
    }

    pub fn url(&self) -> Arc<String> {
        Arc::clone(&self.url)
    }

    /*
     * Chain
     */

    // Genesis

    pub fn genesis_hash(&self) -> Result<H256> {
        self.block_hash(Some(0))
    }

    pub fn genesis_block(&self) -> Result<types::BlockView> {
        self.block_by_number(0)
    }

    // Tip and Current

    pub fn tip_block_number(&self) -> Result<core::BlockNumber> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_tip_block_number(), Default::default())
            .map(::std::convert::Into::into)
            .map(|r: types::BlockNumber| r.0)
    }

    pub fn tip_block_hash(&self) -> Result<H256> {
        self.block_hash(None)
    }

    pub fn tip_header(&self) -> Result<types::HeaderView> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_tip_header(), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn current_epoch(&self) -> Result<types::EpochExt> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_current_epoch(), Default::default())
            .map(::std::convert::Into::into)
    }

    // Block

    pub fn block_hash(&self, number: Option<core::BlockNumber>) -> Result<H256> {
        let cli = self.cli();
        let url = self.url();
        option_to_error!(number, self.tip_block_number()).and_then(move |num| {
            cli.post(&*url)
                .send(
                    Ckb::get_block_hash(types::BlockNumber(num)),
                    Default::default(),
                )
                .map(::std::convert::Into::into)
                .and_then(|r: Option<H256>| r.ok_or_else(Error::none))
        })
    }

    pub fn block_by_hash(&self, hash: H256) -> Result<types::BlockView> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_block(hash), Default::default())
            .map(::std::convert::Into::into)
            .and_then(|r: Option<types::BlockView>| r.ok_or_else(Error::none))
    }

    pub fn block_by_number(&self, number: core::BlockNumber) -> Result<types::BlockView> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_block_by_number(types::BlockNumber(number)),
                Default::default(),
            )
            .map(::std::convert::Into::into)
            .and_then(|r: Option<types::BlockView>| r.ok_or_else(Error::none))
    }

    // Transaction

    pub fn send(&self, tx: types::Transaction) -> Result<H256> {
        self.cli
            .post(&*self.url())
            .send(Ckb::send_transaction(tx), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn transaction(&self, hash: H256) -> Result<types::TransactionWithStatus> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_transaction(hash), Default::default())
            .map(::std::convert::Into::into)
            .and_then(|r: Option<types::TransactionWithStatus>| r.ok_or_else(Error::none))
    }

    // Cell

    pub fn cells_by_lock_hash(
        &self,
        lock_hash: H256,
        from: core::BlockNumber,
        to: core::BlockNumber,
    ) -> Result<Vec<types::CellOutputWithOutPoint>> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_cells_by_lock_hash(
                    lock_hash,
                    types::BlockNumber(from),
                    types::BlockNumber(to),
                ),
                Default::default(),
            )
            .map(::std::convert::Into::into)
    }

    pub fn live_cell(&self, out_point: types::OutPoint) -> Result<types::CellWithStatus> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_live_cell(out_point), Default::default())
            .map(::std::convert::Into::into)
    }

    // Epoch

    pub fn epoch_by_number(&self, number: core::EpochNumber) -> Result<types::EpochExt> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_epoch_by_number(types::EpochNumber(number)),
                Default::default(),
            )
            .map(::std::convert::Into::into)
            .and_then(|r: Option<types::EpochExt>| r.ok_or_else(Error::none))
    }

    /*
     * Pool
     */

    pub fn tx_pool_info(&self) -> Result<types::TxPoolInfo> {
        self.cli
            .post(&*self.url())
            .send(Ckb::tx_pool_info(), Default::default())
            .map(::std::convert::Into::into)
    }

    /*
     * Stats
     */

    pub fn blockchain_info(&self) -> Result<types::ChainInfo> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_blockchain_info(), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn peers_state(&self) -> Result<Vec<types::PeerState>> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_peers_state(), Default::default())
            .map(::std::convert::Into::into)
    }

    /*
     * Net
     */

    pub fn local_node_info(&self) -> Result<types::Node> {
        self.cli
            .post(&*self.url())
            .send(Ckb::local_node_info(), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn peers(&self) -> Result<Vec<types::Node>> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_peers(), Default::default())
            .map(::std::convert::Into::into)
    }

    /*
     * Test
     */

    pub fn add_node(&self, peer_id: String, address: String) -> Result<()> {
        self.cli
            .post(&*self.url())
            .send(Ckb::add_node(peer_id, address), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn remove_node(&self, peer_id: String) -> Result<()> {
        self.cli
            .post(&*self.url())
            .send(Ckb::remove_node(peer_id), Default::default())
            .map(::std::convert::Into::into)
    }

    pub fn process_block_without_verify(&self, block: types::Block) -> Result<Option<H256>> {
        self.cli
            .post(&*self.url())
            .send(Ckb::process_block_without_verify(block), Default::default())
            .map(::std::convert::Into::into)
    }

    /*
     * Experiment
     */

    pub fn dry_run_send(&self, tx: types::Transaction) -> Result<types::DryRunResult> {
        self.cli
            .post(&*self.url())
            .send(Ckb::dry_run_transaction(tx), Default::default())
            .map(::std::convert::Into::into)
    }

    /*
     * Miner
     */

    pub fn block_template(
        &self,
        bytes_limit: Option<u64>,
        proposals_limit: Option<u64>,
        max_version: Option<core::Version>,
    ) -> Result<types::BlockTemplate> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_block_template(
                    bytes_limit.map(types::Unsigned),
                    proposals_limit.map(types::Unsigned),
                    max_version.map(types::Version),
                ),
                Default::default(),
            )
            .map(::std::convert::Into::into)
    }

    pub fn submit_block(&self, work_id: String, block: types::Block) -> Result<H256> {
        self.cli
            .post(&*self.url())
            .send(Ckb::submit_block(work_id, block), Default::default())
            .map(::std::convert::Into::into)
            .and_then(|r: Option<H256>| r.ok_or_else(Error::none))
    }
}
