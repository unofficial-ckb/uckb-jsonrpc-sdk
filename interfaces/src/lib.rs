// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use jsonrpc_sdk_prelude::{jsonrpc_client, jsonrpc_core, serde_json, JsonRpcRequest};

use jsonrpc_types::{
    Block, BlockNumber, BlockTemplate, CellOutputWithOutPoint, CellWithStatus, Cycle, Header, Node,
    OutPoint, Transaction, TransactionWithStatus, TxPoolInfo, TxTrace, Version,
};

pub use bytes;
pub use ckb_core as core;
pub use jsonrpc_types as types;
pub use numext_fixed_hash::{h256, H256};
pub use occupied_capacity::OccupiedCapacity;

jsonrpc_client!(|| {
    pub trait Ckb {
        // Chain
        fn get_block(H256) -> Option<Block>;
        fn get_transaction(H256) -> Option<TransactionWithStatus>;
        fn get_block_hash(BlockNumber) -> Option<H256>;
        fn get_tip_header() -> Header;
        fn get_cells_by_lock_hash(H256, BlockNumber, BlockNumber) -> Vec<CellOutputWithOutPoint>;
        fn get_live_cell(OutPoint) -> CellWithStatus;
        fn get_tip_block_number() -> BlockNumber;
        // Miner
        fn get_block_template(Option<Cycle>, Option<Cycle>, Option<Version>) -> BlockTemplate;
        fn submit_block(String, Block) -> Option<H256>;
        // Net
        fn local_node_info() -> Node;
        fn get_peers() -> Vec<Node>;
        // Pool
        fn send_transaction(Transaction) -> H256;
        fn tx_pool_info() -> TxPoolInfo;
        // Test
        fn add_node(String, String);
        fn enqueue_test_transaction(Transaction) -> H256;
        // Trace
        fn trace_transaction(Transaction) -> H256;
        fn get_transaction_trace(H256) -> Option<Vec<TxTrace>>;
    }
});
