// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use jsonrpc_core::futures::Future;
use uckb_jsonrpc_core::types::{core, fixed, packed, rpc};

use super::HttpClient;
use crate::{
    error::{Error, Result},
    Client,
};

// block on future
macro_rules! b {
    ($self:ident, $method:ident $( ,$param:ident )*) => {{
        b!($self, $method, $( $param, )* )
    }};
    ($self:ident, $method:ident, $( $param:ident, )*) => {{
        let rt = $self.runtime();
        let fut = $self.http()?.$method( $( $param, )* );
        rt.block_on_01(fut)
    }};
}

// convert future
macro_rules! c{
    ($self:ident, $method:ident $( ,$param:expr )*) => {{
        c!($self, $method, $( $param, )* )
    }};
    ($self:ident, $method:ident, $( $param:expr, )*) => {{
        $self.client()
            .$method( $( $param, )* )
            .map_err(Into::into)
    }};
}

macro_rules! coi {
    ($self:ident, $method:ident $( ,$param:expr )*) => {{
        coi!($self, $method, $( $param, )* )
    }};
    ($self:ident, $method:ident, $( $param:expr, )*) => {{
        c!($self, $method, $( $param, )*)
            .map(|res| res.map(Into::into))
    }};
}

macro_rules! ci {
    ($self:ident, $method:ident $( ,$param:expr )*) => {{
        ci!($self, $method, $( $param, )* )
    }};
    ($self:ident, $method:ident, $( $param:expr, )*) => {{
        c!($self, $method, $( $param, )*)
            .map(Into::into)
    }};
}

impl Client {
    // Module Chain
    pub fn get_block(
        &self,
        block_hash: fixed::H256,
        verbosity: Option<u32>,
    ) -> Result<Option<core::BlockView>> {
        b!(self, get_block, block_hash, verbosity)
    }

    pub fn get_block_by_number(
        &self,
        block_number: core::BlockNumber,
        verbosity: Option<u32>,
    ) -> Result<Option<core::BlockView>> {
        b!(self, get_block_by_number, block_number, verbosity)
    }

    pub fn get_header(
        &self,
        block_hash: fixed::H256,
        verbosity: Option<u32>,
    ) -> Result<Option<core::HeaderView>> {
        b!(self, get_header, block_hash, verbosity)
    }

    pub fn get_header_by_number(
        &self,
        block_number: core::BlockNumber,
        verbosity: Option<u32>,
    ) -> Result<Option<core::HeaderView>> {
        b!(self, get_header_by_number, block_number, verbosity)
    }

    pub fn get_transaction(
        &self,
        tx_hash: fixed::H256,
    ) -> Result<Option<rpc::TransactionWithStatus>> {
        b!(self, get_transaction, tx_hash)
    }

    pub fn get_block_hash(&self, block_number: core::BlockNumber) -> Result<Option<fixed::H256>> {
        b!(self, get_block_hash, block_number)
    }

    pub fn get_tip_header(&self, verbosity: Option<u32>) -> Result<core::HeaderView> {
        b!(self, get_tip_header, verbosity)
    }

    pub fn get_live_cell(
        &self,
        out_point: packed::OutPoint,
        with_data: bool,
    ) -> Result<rpc::CellWithStatus> {
        b!(self, get_live_cell, out_point, with_data)
    }

    pub fn get_tip_block_number(&self) -> Result<core::BlockNumber> {
        b!(self, get_tip_block_number)
    }

    pub fn get_current_epoch(&self) -> Result<rpc::EpochView> {
        b!(self, get_current_epoch)
    }

    pub fn get_epoch_by_number(
        &self,
        epoch_number: core::EpochNumber,
    ) -> Result<Option<rpc::EpochView>> {
        b!(self, get_epoch_by_number, epoch_number)
    }

    pub fn get_block_economic_state(
        &self,
        block_hash: fixed::H256,
    ) -> Result<Option<rpc::BlockEconomicState>> {
        b!(self, get_block_economic_state, block_hash)
    }

    pub fn get_transaction_proof(
        &self,
        tx_hashes: Vec<fixed::H256>,
        block_hash: Option<fixed::H256>,
    ) -> Result<rpc::TransactionProof> {
        b!(self, get_transaction_proof, tx_hashes, block_hash)
    }

    pub fn verify_transaction_proof(
        &self,
        tx_proof: rpc::TransactionProof,
    ) -> Result<Vec<fixed::H256>> {
        b!(self, verify_transaction_proof, tx_proof)
    }

    //
    // Module Pool
    //

    pub fn send_transaction(
        &self,
        tx: packed::Transaction,
        outputs_validator: Option<rpc::OutputsValidator>,
    ) -> Result<fixed::H256> {
        b!(self, send_transaction, tx, outputs_validator)
    }

    pub fn tx_pool_info(&self) -> Result<rpc::TxPoolInfo> {
        b!(self, tx_pool_info)
    }

    pub fn clear_tx_pool(&self) -> Result<()> {
        b!(self, clear_tx_pool)
    }

    //
    // Module Miner
    //

    pub fn get_block_template(
        &self,
        bytes_limit: Option<u64>,
        proposals_limit: Option<u64>,
        max_version: Option<core::Version>,
    ) -> Result<rpc::BlockTemplate> {
        b!(
            self,
            get_block_template,
            bytes_limit,
            proposals_limit,
            max_version,
        )
    }

    pub fn submit_block(&self, work_id: String, block: packed::Block) -> Result<fixed::H256> {
        b!(self, submit_block, work_id, block)
    }

    //
    // Module Stats
    //

    pub fn get_blockchain_info(&self) -> Result<rpc::ChainInfo> {
        b!(self, get_blockchain_info)
    }

    //
    // Module Net
    //

    pub fn local_node_info(&self) -> Result<rpc::LocalNode> {
        b!(self, local_node_info)
    }

    pub fn get_peers(&self) -> Result<Vec<rpc::RemoteNode>> {
        b!(self, get_peers)
    }

    pub fn get_banned_addresses(&self) -> Result<Vec<rpc::BannedAddr>> {
        b!(self, get_banned_addresses)
    }

    pub fn clear_banned_addresses(&self) -> Result<()> {
        b!(self, clear_banned_addresses)
    }

    pub fn set_ban(
        &self,
        address: String,
        command: String,
        ban_time: Option<rpc::Timestamp>,
        absolute: Option<bool>,
        reason: Option<String>,
    ) -> Result<()> {
        b!(self, set_ban, address, command, ban_time, absolute, reason)
    }

    pub fn sync_state(&self) -> Result<rpc::SyncState> {
        b!(self, sync_state)
    }

    pub fn set_network_active(&self, state: bool) -> Result<()> {
        b!(self, set_network_active, state)
    }

    pub fn add_node(&self, peer_id: String, address: String) -> Result<()> {
        b!(self, add_node, peer_id, address)
    }

    pub fn remove_node(&self, peer_id: String) -> Result<()> {
        b!(self, remove_node, peer_id)
    }

    pub fn ping_peers(&self) -> Result<()> {
        b!(self, ping_peers)
    }

    //
    // Module Alert
    //

    pub fn send_alert(&self, alert: rpc::Alert) -> Result<()> {
        b!(self, send_alert, alert)
    }

    //
    // Module Experiment
    //

    pub fn dry_run_transaction(&self, tx: packed::Transaction) -> Result<rpc::DryRunResult> {
        b!(self, dry_run_transaction, tx)
    }

    pub fn calculate_dao_maximum_withdraw(
        &self,
        out_point: packed::OutPoint,
        block_hash: fixed::H256,
    ) -> Result<core::Capacity> {
        b!(self, calculate_dao_maximum_withdraw, out_point, block_hash)
    }

    //
    // Module Debug
    //

    pub fn jemalloc_profiling_dump(&self) -> Result<String> {
        b!(self, jemalloc_profiling_dump)
    }

    pub fn update_main_logger(&self, config: rpc::MainLoggerConfig) -> Result<()> {
        b!(self, update_main_logger, config)
    }

    pub fn set_extra_logger(
        &self,
        name: String,
        config_opt: Option<rpc::ExtraLoggerConfig>,
    ) -> Result<()> {
        b!(self, set_extra_logger, name, config_opt)
    }

    //
    // Module IntegrationTest
    //

    pub fn process_block_without_verify(
        &self,
        data: packed::Block,
        broadcast: bool,
    ) -> Result<Option<fixed::H256>> {
        b!(self, process_block_without_verify, data, broadcast)
    }

    pub fn truncate(&self, target_tip_hash: fixed::H256) -> Result<()> {
        b!(self, truncate, target_tip_hash)
    }

    pub fn generate_block(
        &self,
        block_assembler_script: Option<packed::Script>,
        block_assembler_message: Option<packed::Bytes>,
    ) -> Result<fixed::H256> {
        b!(
            self,
            generate_block,
            block_assembler_script,
            block_assembler_message
        )
    }

    pub fn broadcast_transaction(
        &self,
        transaction: packed::Transaction,
        cycles: core::Cycle,
    ) -> Result<fixed::H256> {
        b!(self, broadcast_transaction, transaction, cycles)
    }

    pub fn get_fork_block(&self, hash: fixed::H256) -> Result<Option<rpc::BlockView>> {
        b!(self, get_fork_block, hash)
    }
}

impl HttpClient {
    //
    // Module Chain
    //

    fn get_block(
        &self,
        block_hash: fixed::H256,
        verbosity: Option<u32>,
    ) -> impl Future<Item = Option<core::BlockView>, Error = Error> {
        coi!(self, get_block, block_hash, verbosity.map(Into::into),)
    }

    fn get_block_by_number(
        &self,
        block_number: core::BlockNumber,
        verbosity: Option<u32>,
    ) -> impl Future<Item = Option<core::BlockView>, Error = Error> {
        coi!(
            self,
            get_block_by_number,
            block_number.into(),
            verbosity.map(Into::into)
        )
    }

    fn get_header(
        &self,
        block_hash: fixed::H256,
        verbosity: Option<u32>,
    ) -> impl Future<Item = Option<core::HeaderView>, Error = Error> {
        coi!(self, get_header, block_hash, verbosity.map(Into::into))
    }

    fn get_header_by_number(
        &self,
        block_number: core::BlockNumber,
        verbosity: Option<u32>,
    ) -> impl Future<Item = Option<core::HeaderView>, Error = Error> {
        coi!(
            self,
            get_header_by_number,
            block_number.into(),
            verbosity.map(Into::into)
        )
    }

    fn get_transaction(
        &self,
        tx_hash: fixed::H256,
    ) -> impl Future<Item = Option<rpc::TransactionWithStatus>, Error = Error> {
        c!(self, get_transaction, tx_hash)
    }

    fn get_block_hash(
        &self,
        block_number: core::BlockNumber,
    ) -> impl Future<Item = Option<fixed::H256>, Error = Error> {
        c!(self, get_block_hash, block_number.into())
    }

    fn get_tip_header(
        &self,
        verbosity: Option<u32>,
    ) -> impl Future<Item = core::HeaderView, Error = Error> {
        ci!(self, get_tip_header, verbosity.map(Into::into))
    }

    fn get_live_cell(
        &self,
        out_point: packed::OutPoint,
        with_data: bool,
    ) -> impl Future<Item = rpc::CellWithStatus, Error = Error> {
        c!(self, get_live_cell, out_point.into(), with_data)
    }

    fn get_tip_block_number(&self) -> impl Future<Item = core::BlockNumber, Error = Error> {
        ci!(self, get_tip_block_number)
    }

    fn get_current_epoch(&self) -> impl Future<Item = rpc::EpochView, Error = Error> {
        c!(self, get_current_epoch)
    }

    fn get_epoch_by_number(
        &self,
        epoch_number: core::EpochNumber,
    ) -> impl Future<Item = Option<rpc::EpochView>, Error = Error> {
        c!(self, get_epoch_by_number, epoch_number.into())
    }

    fn get_block_economic_state(
        &self,
        block_hash: fixed::H256,
    ) -> impl Future<Item = Option<rpc::BlockEconomicState>, Error = Error> {
        c!(self, get_block_economic_state, block_hash)
    }

    fn get_transaction_proof(
        &self,
        tx_hashes: Vec<fixed::H256>,
        block_hash: Option<fixed::H256>,
    ) -> impl Future<Item = rpc::TransactionProof, Error = Error> {
        c!(self, get_transaction_proof, tx_hashes, block_hash)
    }

    fn verify_transaction_proof(
        &self,
        tx_proof: rpc::TransactionProof,
    ) -> impl Future<Item = Vec<fixed::H256>, Error = Error> {
        c!(self, verify_transaction_proof, tx_proof)
    }

    //
    // Module Pool
    //

    fn send_transaction(
        &self,
        tx: packed::Transaction,
        outputs_validator: Option<rpc::OutputsValidator>,
    ) -> impl Future<Item = fixed::H256, Error = Error> {
        c!(self, send_transaction, tx.into(), outputs_validator)
    }

    fn tx_pool_info(&self) -> impl Future<Item = rpc::TxPoolInfo, Error = Error> {
        c!(self, tx_pool_info)
    }

    fn clear_tx_pool(&self) -> impl Future<Item = (), Error = Error> {
        c!(self, clear_tx_pool)
    }

    //
    // Module Miner
    //

    fn get_block_template(
        &self,
        bytes_limit: Option<u64>,
        proposals_limit: Option<u64>,
        max_version: Option<core::Version>,
    ) -> impl Future<Item = rpc::BlockTemplate, Error = Error> {
        c!(
            self,
            get_block_template,
            bytes_limit.map(Into::into),
            proposals_limit.map(Into::into),
            max_version.map(Into::into)
        )
    }

    fn submit_block(
        &self,
        work_id: String,
        block: packed::Block,
    ) -> impl Future<Item = fixed::H256, Error = Error> {
        c!(self, submit_block, work_id, block.into())
    }
    //
    // Module Stats
    //

    fn get_blockchain_info(&self) -> impl Future<Item = rpc::ChainInfo, Error = Error> {
        c!(self, get_blockchain_info)
    }

    //
    // Module Net
    //

    fn local_node_info(&self) -> impl Future<Item = rpc::LocalNode, Error = Error> {
        c!(self, local_node_info)
    }

    fn get_peers(&self) -> impl Future<Item = Vec<rpc::RemoteNode>, Error = Error> {
        c!(self, get_peers)
    }

    fn get_banned_addresses(&self) -> impl Future<Item = Vec<rpc::BannedAddr>, Error = Error> {
        c!(self, get_banned_addresses)
    }

    fn clear_banned_addresses(&self) -> impl Future<Item = (), Error = Error> {
        c!(self, clear_banned_addresses)
    }

    fn set_ban(
        &self,
        address: String,
        command: String,
        ban_time: Option<rpc::Timestamp>,
        absolute: Option<bool>,
        reason: Option<String>,
    ) -> impl Future<Item = (), Error = Error> {
        c!(self, set_ban, address, command, ban_time, absolute, reason)
    }

    fn sync_state(&self) -> impl Future<Item = rpc::SyncState, Error = Error> {
        c!(self, sync_state)
    }

    fn set_network_active(&self, state: bool) -> impl Future<Item = (), Error = Error> {
        c!(self, set_network_active, state)
    }

    fn add_node(&self, peer_id: String, address: String) -> impl Future<Item = (), Error = Error> {
        c!(self, add_node, peer_id, address)
    }

    fn remove_node(&self, peer_id: String) -> impl Future<Item = (), Error = Error> {
        c!(self, remove_node, peer_id)
    }

    fn ping_peers(&self) -> impl Future<Item = (), Error = Error> {
        c!(self, ping_peers)
    }

    //
    // Module Alert
    //

    fn send_alert(&self, alert: rpc::Alert) -> impl Future<Item = (), Error = Error> {
        c!(self, send_alert, alert)
    }

    //
    // Module Experiment
    //

    fn dry_run_transaction(
        &self,
        tx: packed::Transaction,
    ) -> impl Future<Item = rpc::DryRunResult, Error = Error> {
        c!(self, dry_run_transaction, tx.into())
    }

    fn calculate_dao_maximum_withdraw(
        &self,
        out_point: packed::OutPoint,
        block_hash: fixed::H256,
    ) -> impl Future<Item = core::Capacity, Error = Error> {
        ci!(
            self,
            calculate_dao_maximum_withdraw,
            out_point.into(),
            block_hash
        )
    }

    //
    // Module Debug
    //

    fn jemalloc_profiling_dump(&self) -> impl Future<Item = String, Error = Error> {
        c!(self, jemalloc_profiling_dump)
    }

    fn update_main_logger(
        &self,
        config: rpc::MainLoggerConfig,
    ) -> impl Future<Item = (), Error = Error> {
        c!(self, update_main_logger, config)
    }

    fn set_extra_logger(
        &self,
        name: String,
        config_opt: Option<rpc::ExtraLoggerConfig>,
    ) -> impl Future<Item = (), Error = Error> {
        c!(self, set_extra_logger, name, config_opt.map(Into::into))
    }

    //
    // Module IntegrationTest
    //

    fn process_block_without_verify(
        &self,
        data: packed::Block,
        broadcast: bool,
    ) -> impl Future<Item = Option<fixed::H256>, Error = Error> {
        c!(self, process_block_without_verify, data.into(), broadcast)
    }

    fn truncate(&self, target_tip_hash: fixed::H256) -> impl Future<Item = (), Error = Error> {
        c!(self, truncate, target_tip_hash)
    }

    fn generate_block(
        &self,
        block_assembler_script: Option<packed::Script>,
        block_assembler_message: Option<packed::Bytes>,
    ) -> impl Future<Item = fixed::H256, Error = Error> {
        c!(
            self,
            generate_block,
            block_assembler_script.map(Into::into),
            block_assembler_message.map(Into::into)
        )
    }

    fn broadcast_transaction(
        &self,
        transaction: packed::Transaction,
        cycles: core::Cycle,
    ) -> impl Future<Item = fixed::H256, Error = Error> {
        c!(
            self,
            broadcast_transaction,
            transaction.into(),
            cycles.into()
        )
    }

    fn get_fork_block(
        &self,
        hash: fixed::H256,
    ) -> impl Future<Item = Option<rpc::BlockView>, Error = Error> {
        coi!(self, get_fork_block, hash)
    }
}
