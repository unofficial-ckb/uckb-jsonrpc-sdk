// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(clippy::unused_unit)]

use jsonrpc_sdk_prelude::{jsonrpc_client, jsonrpc_core, serde_json, JsonRpcRequest};

pub use ckb_core::BlockNumber;
pub use jsonrpc_types::{
    Block, BlockTemplate, CellOutputWithOutPoint, CellWithStatus, Header, Node, OutPoint,
    Transaction, TxTrace,
};
pub use numext_fixed_hash::H256;

jsonrpc_client!(|| {
    pub trait Ckb {
        // Chain
        fn get_block(H256) -> Option<Block>;
        fn get_transaction(H256) -> Option<Transaction>;
        fn get_block_hash(String) -> Option<H256>;
        fn get_tip_header() -> Header;
        fn get_cells_by_lock_hash(H256, String, String) -> Vec<CellOutputWithOutPoint>;
        fn get_live_cell(OutPoint) -> CellWithStatus;
        fn get_tip_block_number() -> BlockNumber;
        // Miner
        fn get_block_template(Option<String>, Option<String>, Option<String>) -> BlockTemplate;
        fn submit_block(String, Block) -> Option<H256>;
        // Net
        fn local_node_info() -> Node;
        fn get_peers() -> Vec<Node>;
        // Pool
        fn send_transaction(Transaction) -> H256;
        fn get_pool_transaction(H256) -> Option<Transaction>;
        // Test
        fn add_node(String, String) -> ();
        // Trace
        fn trace_transaction(Transaction) -> H256;
        fn get_transaction_trace(H256) -> Option<Vec<TxTrace>>;
    }
});
