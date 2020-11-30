// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use http::gen_client::Client as HttpClient;
pub use tcp::gen_client::Client as TcpClient;

pub(crate) mod tcp {
    use crate::types::rpc::*;
    use jsonrpc_derive::rpc;

    #[rpc(client)]
    pub trait TcpRpc {
        // Module Subscription
        type Metadata;
        #[pubsub(subscription = "subscribe", subscribe, name = "subscribe")]
        fn subscribe(&self, meta: Self::Metadata, subscriber: Subscriber<String>, topic: Topic);
        #[pubsub(subscription = "subscribe", unsubscribe, name = "unsubscribe")]
        fn unsubscribe(&self, meta: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool>;
    }
}

pub(crate) mod http {
    use crate::types::{fixed::H256, rpc::*};
    use jsonrpc_derive::rpc;

    #[rpc(client)]
    pub trait HttpRpc {
        // Module Chain
        #[rpc(name = "get_block")]
        fn get_block(
            &self,
            block_hash: H256,
            verbosity: Option<Uint32>,
        ) -> Result<Option<BlockView>>;
        #[rpc(name = "get_block_by_number")]
        fn get_block_by_number(
            &self,
            block_number: BlockNumber,
            verbosity: Option<Uint32>,
        ) -> Result<Option<BlockView>>;
        #[rpc(name = "get_header")]
        fn get_header(
            &self,
            block_hash: H256,
            verbosity: Option<Uint32>,
        ) -> Result<Option<HeaderView>>;
        #[rpc(name = "get_header_by_number")]
        fn get_header_by_number(
            &self,
            block_number: BlockNumber,
            verbosity: Option<Uint32>,
        ) -> Result<Option<HeaderView>>;
        #[rpc(name = "get_transaction")]
        fn get_transaction(&self, tx_hash: H256) -> Result<Option<TransactionWithStatus>>;
        #[rpc(name = "get_block_hash")]
        fn get_block_hash(&self, block_number: BlockNumber) -> Result<Option<H256>>;
        #[rpc(name = "get_tip_header")]
        fn get_tip_header(&self, verbosity: Option<Uint32>) -> Result<HeaderView>;
        #[rpc(name = "get_live_cell")]
        fn get_live_cell(&self, out_point: OutPoint, with_data: bool) -> Result<CellWithStatus>;
        #[rpc(name = "get_tip_block_number")]
        fn get_tip_block_number(&self) -> Result<BlockNumber>;
        #[rpc(name = "get_current_epoch")]
        fn get_current_epoch(&self) -> Result<EpochView>;
        #[rpc(name = "get_epoch_by_number")]
        fn get_epoch_by_number(&self, epoch_number: EpochNumber) -> Result<Option<EpochView>>;
        #[rpc(name = "get_block_economic_state")]
        fn get_block_economic_state(&self, block_hash: H256) -> Result<Option<BlockEconomicState>>;
        #[rpc(name = "get_transaction_proof")]
        fn get_transaction_proof(
            &self,
            tx_hashes: Vec<H256>,
            block_hash: Option<H256>,
        ) -> Result<TransactionProof>;
        #[rpc(name = "verify_transaction_proof")]
        fn verify_transaction_proof(&self, tx_proof: TransactionProof) -> Result<Vec<H256>>;
        #[rpc(name = "get_fork_block")]
        fn get_fork_block(
            &self,
            block_hash: H256,
            verbosity: Option<Uint32>,
        ) -> Result<Option<BlockView>>;
        // Module Pool
        #[rpc(name = "send_transaction")]
        fn send_transaction(
            &self,
            tx: Transaction,
            outputs_validator: Option<OutputsValidator>,
        ) -> Result<H256>;
        #[rpc(name = "tx_pool_info")]
        fn tx_pool_info(&self) -> Result<TxPoolInfo>;
        #[rpc(name = "clear_tx_pool")]
        fn clear_tx_pool(&self) -> Result<()>;
        // Module Miner
        #[rpc(name = "get_block_template")]
        fn get_block_template(
            &self,
            bytes_limit: Option<Uint64>,
            proposals_limit: Option<Uint64>,
            max_version: Option<crate::types::rpc::Version>,
        ) -> Result<BlockTemplate>;
        #[rpc(name = "submit_block")]
        fn submit_block(&self, work_id: String, block: Block) -> Result<H256>;
        // Module Stats
        #[rpc(name = "get_blockchain_info")]
        fn get_blockchain_info(&self) -> Result<ChainInfo>;
        // Module Net
        #[rpc(name = "local_node_info")]
        fn local_node_info(&self) -> Result<LocalNode>;
        #[rpc(name = "get_peers")]
        fn get_peers(&self) -> Result<Vec<RemoteNode>>;
        #[rpc(name = "get_banned_addresses")]
        fn get_banned_addresses(&self) -> Result<Vec<BannedAddr>>;
        #[rpc(name = "clear_banned_addresses")]
        fn clear_banned_addresses(&self) -> Result<()>;
        #[rpc(name = "set_ban")]
        fn set_ban(
            &self,
            address: String,
            command: String,
            ban_time: Option<Timestamp>,
            absolute: Option<bool>,
            reason: Option<String>,
        ) -> Result<()>;
        #[rpc(name = "sync_state")]
        fn sync_state(&self) -> Result<SyncState>;
        #[rpc(name = "set_network_active")]
        fn set_network_active(&self, state: bool) -> Result<()>;
        #[rpc(name = "add_node")]
        fn add_node(&self, peer_id: String, address: String) -> Result<()>;
        #[rpc(name = "remove_node")]
        fn remove_node(&self, peer_id: String) -> Result<()>;
        #[rpc(name = "ping_peers")]
        fn ping_peers(&self) -> Result<()>;
        // Module Alert
        #[rpc(name = "send_alert")]
        fn send_alert(&self, alert: Alert) -> Result<()>;
        // Module Experiment
        #[rpc(name = "dry_run_transaction")]
        fn dry_run_transaction(&self, tx: Transaction) -> Result<DryRunResult>;
        #[rpc(name = "calculate_dao_maximum_withdraw")]
        fn calculate_dao_maximum_withdraw(
            &self,
            out_point: OutPoint,
            block_hash: H256,
        ) -> Result<Capacity>;
        // Module Debug
        #[rpc(name = "jemalloc_profiling_dump")]
        fn jemalloc_profiling_dump(&self) -> Result<String>;
        #[rpc(name = "update_main_logger")]
        fn update_main_logger(&self, config: MainLoggerConfig) -> Result<()>;
        #[rpc(name = "set_extra_logger")]
        fn set_extra_logger(
            &self,
            name: String,
            config_opt: Option<ExtraLoggerConfig>,
        ) -> Result<()>;
        // Module IntegrationTest
        #[rpc(name = "process_block_without_verify")]
        fn process_block_without_verify(
            &self,
            data: Block,
            broadcast: bool,
        ) -> Result<Option<H256>>;
        #[rpc(name = "truncate")]
        fn truncate(&self, target_tip_hash: H256) -> Result<()>;
        #[rpc(name = "generate_block")]
        fn generate_block(
            &self,
            block_assembler_script: Option<Script>,
            block_assembler_message: Option<JsonBytes>,
        ) -> Result<H256>;
        #[rpc(name = "broadcast_transaction")]
        fn broadcast_transaction(&self, transaction: Transaction, cycles: Cycle) -> Result<H256>;
    }
}
