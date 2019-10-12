// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{convert::Into, sync::Arc};

use futures::{future, Future};

use jsonrpc_sdk_client::r#async::Client;
use jsonrpc_sdk_prelude::Error;

use ckb_jsonrpc_interfaces::{
    types::{core, packed, rpc, H256},
    Ckb,
};

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
}

// Query Chain
impl CkbClient {
    // Genesis

    pub fn genesis_hash(&self) -> impl Future<Item = H256, Error = Error> {
        self.block_hash(Some(0))
    }

    pub fn genesis_block(&self) -> impl Future<Item = core::BlockView, Error = Error> {
        self.block_by_number(0)
    }

    pub fn genesis_header(&self) -> impl Future<Item = core::HeaderView, Error = Error> {
        self.header_by_number(0)
    }

    // Tip and Current

    pub fn tip_number(&self) -> impl Future<Item = core::BlockNumber, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_tip_block_number(), Default::default())
            .map(Into::<rpc::BlockNumber>::into)
            .map(Into::into)
    }

    pub fn tip_header(&self) -> impl Future<Item = core::HeaderView, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_tip_header(), Default::default())
            .map(Into::<rpc::HeaderView>::into)
            .map(Into::into)
    }

    pub fn tip_epoch(&self) -> impl Future<Item = rpc::EpochView, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_current_epoch(), Default::default())
            .map(Into::into)
    }

    pub fn tip_hash(&self) -> impl Future<Item = H256, Error = Error> {
        self.block_hash(None)
    }

    // Block and Header

    pub fn block_hash(
        &self,
        number: Option<core::BlockNumber>,
    ) -> impl Future<Item = H256, Error = Error> {
        let cli = self.cli();
        let url = self.url();
        option_to_future!(number, self.tip_number()).and_then(move |num| {
            cli.post(&*url)
                .send(Ckb::get_block_hash(num.into()), Default::default())
                .map(Into::into)
                .and_then(|r: Option<H256>| r.ok_or_else(Error::none))
        })
    }

    pub fn block_by_hash(&self, hash: H256) -> impl Future<Item = core::BlockView, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_block(hash), Default::default())
            .map(Into::into)
            .and_then(|r: Option<rpc::BlockView>| r.ok_or_else(Error::none))
            .map(Into::into)
    }

    pub fn block_by_number(
        &self,
        number: core::BlockNumber,
    ) -> impl Future<Item = core::BlockView, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_block_by_number(number.into()), Default::default())
            .map(Into::into)
            .and_then(|r: Option<rpc::BlockView>| r.ok_or_else(Error::none))
            .map(Into::into)
    }

    pub fn header_by_hash(
        &self,
        hash: H256,
    ) -> impl Future<Item = core::HeaderView, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_header(hash), Default::default())
            .map(Into::into)
            .and_then(|r: Option<rpc::HeaderView>| r.ok_or_else(Error::none))
            .map(Into::into)
    }

    pub fn header_by_number(
        &self,
        number: core::BlockNumber,
    ) -> impl Future<Item = core::HeaderView, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_header_by_number(number.into()), Default::default())
            .map(Into::into)
            .and_then(|r: Option<rpc::HeaderView>| r.ok_or_else(Error::none))
            .map(Into::into)
    }

    // Epoch

    pub fn epoch_by_number(
        &self,
        number: core::EpochNumber,
    ) -> impl Future<Item = rpc::EpochView, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_epoch_by_number(number.into()), Default::default())
            .map(Into::into)
            .and_then(|r: Option<rpc::EpochView>| r.ok_or_else(Error::none))
    }

    // Transaction

    pub fn transaction(
        &self,
        hash: H256,
    ) -> impl Future<Item = (core::TransactionView, rpc::TxStatus), Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_transaction(hash), Default::default())
            .map(Into::into)
            .and_then(|r: Option<rpc::TransactionWithStatus>| r.ok_or_else(Error::none))
            .map(|txw| {
                let tx: packed::Transaction = txw.transaction.inner.into();
                (tx.into_view(), txw.tx_status)
            })
    }

    // Cell

    pub fn cells_by_lock_hash(
        &self,
        lock_hash: H256,
        from: core::BlockNumber,
        to: core::BlockNumber,
    ) -> impl Future<Item = Vec<rpc::CellOutputWithOutPoint>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_cells_by_lock_hash(lock_hash, from.into(), to.into()),
                Default::default(),
            )
            .map(Into::into)
    }

    pub fn live_cell(
        &self,
        out_point: packed::OutPoint,
        with_data: bool,
    ) -> impl Future<Item = rpc::CellWithStatus, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_live_cell(out_point.into(), with_data),
                Default::default(),
            )
            .map(Into::into)
    }

    // Other

    pub fn get_cellbase_output_capacity_details(
        &self,
        block_hash: H256,
    ) -> impl Future<Item = Option<rpc::BlockReward>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_cellbase_output_capacity_details(block_hash),
                Default::default(),
            )
            .map(Into::into)
    }
}

// Query Others
impl CkbClient {
    // Pool

    pub fn tx_pool_info(&self) -> impl Future<Item = rpc::TxPoolInfo, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::tx_pool_info(), Default::default())
            .map(Into::into)
    }

    // Indexer

    pub fn live_cells_by_lock_hash(
        &self,
        lock_hash: H256,
        page: u64,
        per_page: u64,
        reverse_order: Option<bool>,
    ) -> impl Future<Item = Vec<rpc::LiveCell>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_live_cells_by_lock_hash(
                    lock_hash,
                    page.into(),
                    per_page.into(),
                    reverse_order,
                ),
                Default::default(),
            )
            .map(Into::into)
    }

    pub fn transactions_by_lock_hash(
        &self,
        lock_hash: H256,
        page: u64,
        per_page: u64,
        reverse_order: Option<bool>,
    ) -> impl Future<Item = Vec<rpc::CellTransaction>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_transactions_by_lock_hash(
                    lock_hash,
                    page.into(),
                    per_page.into(),
                    reverse_order,
                ),
                Default::default(),
            )
            .map(Into::into)
    }

    pub fn get_lock_hash_index_states(
        &self,
    ) -> impl Future<Item = Vec<rpc::LockHashIndexState>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_lock_hash_index_states(), Default::default())
            .map(Into::into)
    }

    // Stats

    pub fn blockchain_info(&self) -> impl Future<Item = rpc::ChainInfo, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_blockchain_info(), Default::default())
            .map(Into::into)
    }

    pub fn peers_state(&self) -> impl Future<Item = Vec<rpc::PeerState>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_peers_state(), Default::default())
            .map(Into::into)
    }

    // Net

    pub fn local_node_info(&self) -> impl Future<Item = rpc::Node, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::local_node_info(), Default::default())
            .map(Into::into)
    }

    pub fn peers(&self) -> impl Future<Item = Vec<rpc::Node>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_peers(), Default::default())
            .map(Into::into)
    }

    pub fn banned_addresses(&self) -> impl Future<Item = Vec<rpc::BannedAddr>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::get_banned_addresses(), Default::default())
            .map(Into::into)
    }

    // Miner

    pub fn block_template(
        &self,
        bytes_limit: Option<u64>,
        proposals_limit: Option<u64>,
        max_version: Option<core::Version>,
    ) -> impl Future<Item = rpc::BlockTemplate, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::get_block_template(
                    bytes_limit.map(Into::into),
                    proposals_limit.map(Into::into),
                    max_version.map(Into::into),
                ),
                Default::default(),
            )
            .map(Into::into)
    }
}

// Submit Chain
impl CkbClient {
    // Pool

    pub fn send_transaction(
        &self,
        tx: packed::Transaction,
    ) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::send_transaction(tx.into()), Default::default())
            .map(Into::into)
    }

    // Miner

    pub fn submit_block(
        &self,
        work_id: String,
        block: packed::Block,
    ) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::submit_block(work_id, block.into()), Default::default())
            .map(Into::into)
    }
}

// Submit Others
impl CkbClient {
    // Indexer

    pub fn index_lock_hash(
        &self,
        lock_hash: H256,
        index_from: Option<core::BlockNumber>,
    ) -> impl Future<Item = rpc::LockHashIndexState, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::index_lock_hash(lock_hash, index_from.map(Into::into)),
                Default::default(),
            )
            .map(Into::into)
    }

    pub fn deindex_lock_hash(&self, hash: H256) -> impl Future<Item = (), Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::deindex_lock_hash(hash), Default::default())
            .map(Into::into)
    }

    // Net

    pub fn set_ban(
        &self,
        address: String,
        command: String,
        ban_ms: Option<u64>,
        absolute: Option<bool>,
        reason: Option<String>,
    ) -> impl Future<Item = (), Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::set_ban(address, command, ban_ms.map(Into::into), absolute, reason),
                Default::default(),
            )
            .map(Into::into)
    }

    // Alert

    pub fn send_alert(&self, alert: packed::Alert) -> impl Future<Item = (), Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::send_alert(alert.into()), Default::default())
            .map(Into::into)
    }
}

// Unstable
impl CkbClient {
    // Test

    pub fn add_node(
        &self,
        peer_id: String,
        address: String,
    ) -> impl Future<Item = (), Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::add_node(peer_id, address), Default::default())
            .map(Into::into)
    }

    pub fn remove_node(&self, peer_id: String) -> impl Future<Item = (), Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::remove_node(peer_id), Default::default())
            .map(Into::into)
    }

    pub fn process_block_without_verify(
        &self,
        block: packed::Block,
    ) -> impl Future<Item = Option<H256>, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::process_block_without_verify(block.into()),
                Default::default(),
            )
            .map(Into::into)
    }

    pub fn broadcast_transaction(
        &self,
        tx: packed::Transaction,
        cycle: core::Cycle,
    ) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::broadcast_transaction(tx.into(), cycle.into()),
                Default::default(),
            )
            .map(Into::into)
    }

    // Experiment

    pub fn compute_transaction_hash(
        &self,
        tx: packed::Transaction,
    ) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::compute_transaction_hash(tx.into()), Default::default())
            .map(Into::into)
    }

    pub fn compute_script_hash(
        &self,
        script: packed::Script,
    ) -> impl Future<Item = H256, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::compute_script_hash(script.into()), Default::default())
            .map(Into::into)
    }

    pub fn dry_run_transaction(
        &self,
        tx: packed::Transaction,
    ) -> impl Future<Item = rpc::DryRunResult, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(Ckb::dry_run_transaction(tx.into()), Default::default())
            .map(Into::into)
    }

    pub fn calculate_dao_maximum_withdraw(
        &self,
        out_point: packed::OutPoint,
        hash: H256,
    ) -> impl Future<Item = core::Capacity, Error = Error> {
        self.cli
            .post(&*self.url())
            .send(
                Ckb::calculate_dao_maximum_withdraw(out_point.into(), hash),
                Default::default(),
            )
            .map(Into::<rpc::Capacity>::into)
            .map(Into::into)
    }
}
