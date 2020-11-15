// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{fmt, result};

use jsonrpc_core as rpc;
use jsonrpc_core_client as cli;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("client error: TCP client is not activated")]
    NoTcpClient,
    #[error("client error: HTTP client is not activated")]
    NoHttpClient,

    #[error("tcp client error: {0}")]
    TcpClient(String),
    #[error("http client error: {0}")]
    HttpClient(String),

    #[error("rpc error: {0}")]
    RpcError(cli::RpcError),
}

pub type Result<T> = result::Result<T, Error>;

impl From<cli::RpcError> for Error {
    fn from(error: cli::RpcError) -> Self {
        Self::RpcError(error)
    }
}

impl From<rpc::Error> for Error {
    fn from(error: rpc::Error) -> Self {
        Self::RpcError(error.into())
    }
}

impl Error {
    pub fn runtime<T: fmt::Display>(inner: T) -> Self {
        Self::Runtime(inner.to_string())
    }

    pub fn tcp_client<T: fmt::Display>(inner: T) -> Self {
        Self::TcpClient(inner.to_string())
    }

    pub fn http_client<T: fmt::Display>(inner: T) -> Self {
        Self::HttpClient(inner.to_string())
    }

    pub fn rpc_invalid_params<T: fmt::Display>(inner: T) -> Self {
        rpc::Error {
            code: rpc::ErrorCode::InvalidParams,
            message: inner.to_string(),
            data: None,
        }
        .into()
    }

    pub fn rpc_other<T: fmt::Display>(inner: T) -> Self {
        rpc::Error {
            code: rpc::ErrorCode::InternalError,
            message: inner.to_string(),
            data: None,
        }
        .into()
    }
}
