// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{net::SocketAddr, sync::Arc};

use jsonrpc_server_utils::tokio::runtime::Runtime as RawRuntime01;
use parking_lot::RwLock;
use tokio::runtime::Runtime as RawRuntime;
use url::Url;

use crate::{
    error::{Error, Result},
    runtime::Runtime,
};

mod http;
mod tcp;

use self::{http::HttpClient, tcp::TcpClient};

pub struct Client {
    runtime: Runtime,
    tcp: Option<TcpClient>,
    http: Option<HttpClient>,
}

impl Client {
    pub fn new(rt: Arc<RawRuntime>, rt01: Arc<RwLock<RawRuntime01>>) -> Self {
        log::info!("create a new client");
        Self {
            runtime: Runtime::new(rt, rt01),
            tcp: None,
            http: None,
        }
    }

    fn runtime(&self) -> Runtime {
        self.runtime.clone()
    }

    fn tcp(&self) -> Result<&TcpClient> {
        self.tcp.as_ref().ok_or(Error::NoTcpClient)
    }

    fn http(&self) -> Result<&HttpClient> {
        self.http.as_ref().ok_or(Error::NoHttpClient)
    }

    pub fn enable_tcp(&mut self, addr: &SocketAddr) -> Result<&mut Self> {
        log::info!("enable tcp client");
        if self.tcp.is_none() {
            self.tcp = Some(TcpClient::new(self.runtime(), addr)?);
        }
        Ok(self)
    }

    pub fn enable_http(&mut self, url: &Url) -> Result<&mut Self> {
        log::info!("enable http client");
        if self.http.is_none() {
            self.http = Some(HttpClient::new(self.runtime(), url)?);
        }
        Ok(self)
    }
}
