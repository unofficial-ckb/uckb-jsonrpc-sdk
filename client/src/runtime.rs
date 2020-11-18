// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{future::Future, result, sync::Arc};

use jsonrpc_server_utils::tokio::{prelude::Future as Future01, runtime::Runtime as RawRuntime01};
use parking_lot::RwLock;
use tokio::{runtime::Runtime as RawRuntime, task::JoinHandle};

#[derive(Clone)]
pub(crate) struct Runtime {
    core: Arc<RwLock<RawRuntime>>,
    legacy_support: Arc<RwLock<RawRuntime01>>,
}

impl Runtime {
    pub(crate) fn new(
        core: Arc<RwLock<RawRuntime>>,
        legacy_support: Arc<RwLock<RawRuntime01>>,
    ) -> Self {
        Self {
            core,
            legacy_support,
        }
    }

    pub(crate) fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        log::trace!("block on a future");
        self.core.read().block_on(future)
    }

    pub(crate) fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        log::trace!("spawn a future");
        self.core.read().spawn(future)
    }

    pub(crate) fn block_on_01<F>(&self, future: F) -> result::Result<F::Item, F::Error>
    where
        F: Future01 + Send + 'static,
        F::Item: Send + 'static,
        F::Error: Send + 'static,
    {
        log::trace!("block on a legacy future");
        self.legacy_support.write().block_on(future)
    }

    pub(crate) fn spawn_01<F>(&self, future: F)
    where
        F: Future01<Item = (), Error = ()> + Send + 'static,
        F::Item: Send,
        F::Error: Send,
    {
        log::trace!("spawn a legacy future");
        self.legacy_support.write().spawn(future);
    }
}
