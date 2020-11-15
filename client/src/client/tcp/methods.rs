// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::result;

use futures::{compat::Stream01CompatExt as _, StreamExt as _};
use jsonrpc_server_utils::tokio::prelude::IntoFuture as _;
use tokio::sync::mpsc;
use uckb_jsonrpc_core::types::rpc;

use super::TcpClient;
use crate::{
    error::{Error, Result},
    Client, Runtime,
};

impl Client {
    pub fn subscribe_new_tip_header<F>(&self, func: F) -> Result<()>
    where
        F: Fn(&str) -> result::Result<(), ()> + 'static + Send,
    {
        self.subscribe(rpc::Topic::NewTipHeader, func)
    }

    pub fn subscribe_new_tip_block<F>(&self, func: F) -> Result<()>
    where
        F: Fn(&str) -> result::Result<(), ()> + 'static + Send,
    {
        self.subscribe(rpc::Topic::NewTipBlock, func)
    }

    pub fn subscribe_new_transaction<F>(&self, func: F) -> Result<()>
    where
        F: Fn(&str) -> result::Result<(), ()> + 'static + Send,
    {
        self.subscribe(rpc::Topic::NewTransaction, func)
    }

    fn subscribe<F>(&self, topic: rpc::Topic, func: F) -> Result<()>
    where
        F: Fn(&str) -> result::Result<(), ()> + 'static + Send,
    {
        log::debug!("client subscribe {:?}", topic);
        self.tcp()?.subscribe(self.runtime(), topic, func)
    }
}

impl TcpClient {
    pub(super) fn subscribe<F>(&self, rt: Runtime, topic: rpc::Topic, func: F) -> Result<()>
    where
        F: Fn(&str) -> result::Result<(), ()> + 'static + Send,
    {
        // remove the previous subscription
        if let Some(sender) = self.sess.write().remove(&topic) {
            log::warn!("tcp subscribe {:?} already existed, replace it", topic);
            rt.block_on(sender.send(())).map_err(|_| {
                log::error!(
                    "tcp subscribe {:?} failed to remove the previous subscription",
                    topic,
                );
                Error::tcp_client("failed to remove the previous subscription")
            })?;
        }
        // setup a new subscription
        log::trace!("tcp subscribe {:?}", topic);
        let fut_subscribe = self.client().subscribe(topic).into_future();
        let stream_01 = rt.block_on_01(fut_subscribe).map_err(move |err| {
            log::error!("failed to subscribe {:?} since {}", topic, err);
            Error::tcp_client(err)
        })?;
        log::trace!("tcp subscribe {:?} is ok", topic);
        let (sender, mut receiver) = mpsc::channel(1);
        let mut stream = stream_01.compat();
        let fut = async move {
            loop {
                tokio::select! {
                    _ = receiver.recv() => {
                        log::trace!("tcp subscribe {:?} remove", topic);
                        break;
                    }
                    Some(resp) = stream.next() => {
                        match resp {
                            Ok(msg) => {
                                log::trace!("tcp subscribe {:?} receive {}", topic, msg);
                                if func(&msg).is_err() {
                                    break;
                                }
                            }
                            Err(err) => {
                                log::warn!("tcp subscribe {:?} got an error {}", topic, err);
                                break;
                            }
                        }
                    },
                    else => {
                        log::warn!("tcp subscribe {:?} is broken", topic);
                        break;
                    },
                }
            }
            drop(stream);
        };
        rt.spawn(fut);
        // save the subscription and return
        self.sess.write().insert(topic, sender);
        Ok(())
    }
}
