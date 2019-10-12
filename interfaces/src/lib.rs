// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use jsonrpc_sdk_prelude::{jsonrpc_core, jsonrpc_interfaces, serde_json, JsonRpcRequest};

use ckb_jsonrpc_types::*;
use ckb_types::H256;

// Re-export all required crates from [CKB](https://github.com/nervosnetwork/ckb).
pub use ckb_crypto::secp as secp256k1;
pub use ckb_hash as blake2b;
pub mod types {
    pub use ckb_jsonrpc_types as rpc;
    pub use ckb_types::*;
}

jsonrpc_interfaces!(|| {
    pub trait Ckb {
        // Chain
        fn get_block(H256) -> Option<BlockView>;
        fn get_block_by_number(BlockNumber) -> Option<BlockView>;
        fn get_header(H256) -> Option<HeaderView>;
        fn get_header_by_number(BlockNumber) -> Option<HeaderView>;
        fn get_transaction(H256) -> Option<TransactionWithStatus>;
        fn get_block_hash(BlockNumber) -> Option<H256>;
        fn get_tip_header() -> HeaderView;
        fn get_cells_by_lock_hash(H256, BlockNumber, BlockNumber) -> Vec<CellOutputWithOutPoint>;
        fn get_live_cell(OutPoint, bool) -> CellWithStatus;
        fn get_tip_block_number() -> BlockNumber;
        fn get_current_epoch() -> EpochView;
        fn get_epoch_by_number(EpochNumber) -> Option<EpochView>;
        fn get_cellbase_output_capacity_details(H256) -> Option<BlockReward>;
        // Pool
        fn send_transaction(Transaction) -> H256;
        fn tx_pool_info() -> TxPoolInfo;
        // Indexer
        fn get_live_cells_by_lock_hash(H256, Uint64, Uint64, Option<bool>) -> Vec<LiveCell>;
        fn get_transactions_by_lock_hash(
            H256,
            Uint64,
            Uint64,
            Option<bool>,
        ) -> Vec<CellTransaction>;
        fn index_lock_hash(H256, Option<BlockNumber>) -> LockHashIndexState;
        fn deindex_lock_hash(H256);
        fn get_lock_hash_index_states() -> Vec<LockHashIndexState>;
        // Stats
        fn get_blockchain_info() -> ChainInfo;
        fn get_peers_state() -> Vec<PeerState>;
        // Net
        fn local_node_info() -> Node;
        fn get_peers() -> Vec<Node>;
        fn get_banned_addresses() -> Vec<BannedAddr>;
        fn set_ban(String, String, Option<Timestamp>, Option<bool>, Option<String>);
        // Alert
        fn send_alert(Alert);
        // Miner
        fn get_block_template(Option<Uint64>, Option<Uint64>, Option<Version>) -> BlockTemplate;
        fn submit_block(String, Block) -> H256;
        // Test
        fn add_node(String, String);
        fn remove_node(String);
        fn process_block_without_verify(Block) -> Option<H256>;
        fn broadcast_transaction(Transaction, Cycle) -> H256;
        // Experiment
        fn compute_transaction_hash(Transaction) -> H256;
        fn compute_script_hash(Script) -> H256;
        fn dry_run_transaction(Transaction) -> DryRunResult;
        fn calculate_dao_maximum_withdraw(OutPoint, H256) -> Capacity;
    }
});
