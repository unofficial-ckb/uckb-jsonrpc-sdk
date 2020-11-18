// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{collections::HashMap, net::SocketAddr};

use jsonrpc_core_client::{transports::duplex, RpcError};
use jsonrpc_server_utils::{
    codecs::StreamCodec,
    tokio::{
        codec::Decoder as _,
        net::TcpStream,
        prelude::{Future as _, Sink as _, Stream as _},
    },
};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use uckb_jsonrpc_core::{client::TcpClient as RawTcpClient, types::rpc};

use crate::{
    error::{Error, Result},
    runtime::Runtime,
};

mod methods;

pub(super) struct TcpClient {
    client: RawTcpClient,
    sess: RwLock<HashMap<rpc::Topic, mpsc::Sender<()>>>,
}

impl Drop for TcpClient {
    fn drop(&mut self) {
        for (topic, sender) in self.sess.write().drain() {
            log::trace!("tcp subscribe {:?} drop", topic);
            let _result = sender.send(());
        }
    }
}

impl TcpClient {
    pub(super) fn new(rt: Runtime, addr: &SocketAddr) -> Result<Self> {
        log::trace!("initialize a tcp client to connect {}", addr);
        let fut_conn = TcpStream::connect(addr).map(|stream| {
            log::trace!("successfully connect via {}", stream.local_addr().unwrap());
            stream
        });
        let stream = rt.block_on_01(fut_conn).map_err(Error::tcp_client)?;
        let (sink, stream) = StreamCodec::stream_incoming().framed(stream).split();
        let sink = sink.sink_map_err(|e| RpcError::Other(e.into()));
        let stream = stream.map_err(|e| RpcError::Other(e.into()));
        let (rpc_client, sender) = duplex(sink, stream);
        let client = RawTcpClient::from(sender);
        rt.spawn_01(rpc_client.map_err(|_| ()));
        let sess = RwLock::new(HashMap::new());
        Ok(Self { client, sess })
    }

    pub(super) fn client(&self) -> RawTcpClient {
        self.client.clone()
    }
}
