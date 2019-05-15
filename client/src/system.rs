// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ckb_jsonrpc_interfaces::{blake2b, core, secp256k1, types, H256};

const DEFAULT_PERSONAL: &str = "ckb-default-hash";

pub fn blake2b_160(pubkey: &secp256k1::Pubkey) -> [u8; 32] {
    let mut hash = [0u8; 32];
    let mut blake2b = blake2b::Blake2bBuilder::new(32)
        .personal(DEFAULT_PERSONAL.as_bytes())
        .build();
    blake2b.update(&pubkey.serialize());
    blake2b.finalize(&mut hash);
    hash
}

pub fn calculate_arg(pubkey: &secp256k1::Pubkey) -> [u8; 20] {
    let arg = blake2b_160(pubkey);
    let mut ret = [0u8; 20];
    ret.copy_from_slice(&arg[..20]);
    ret
}

pub fn calculate_code_hash(bytes: &[u8]) -> H256 {
    H256::from(&blake2b::blake2b_256(bytes))
}

pub fn calculate_code_hash_and_dep(
    transaction: &types::TransactionView,
    index: u32,
) -> (H256, core::transaction::OutPoint) {
    let tx_hash = transaction.hash.to_owned();
    let bytes = transaction.inner.outputs[index as usize].data.as_bytes();
    let cell_out_point = core::transaction::CellOutPoint { tx_hash, index };
    let dep = core::transaction::OutPoint {
        cell: Some(cell_out_point),
        block_hash: None,
    };
    let code_hash = calculate_code_hash(bytes);
    (code_hash, dep)
}

pub fn calculate_secp256k1_code_hash_and_dep(
    genesis_block: &types::BlockView,
) -> (H256, core::transaction::OutPoint) {
    let tx = &genesis_block.transactions[0];
    calculate_code_hash_and_dep(tx, 1)
}

pub fn calculate_witness(
    privkey: &secp256k1::Privkey,
    tx_hash: &H256,
) -> core::transaction::Witness {
    let pubkey = privkey.pubkey().unwrap().serialize();
    let signature = privkey.sign_recoverable(tx_hash).unwrap();
    let signature_der = signature.serialize_der();
    let signature_size = (signature_der.len() as u64).to_le_bytes().to_vec();
    vec![pubkey.into(), signature_der.into(), signature_size.into()]
}
