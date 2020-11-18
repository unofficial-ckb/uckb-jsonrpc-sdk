// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use jsonrpc_core_client::transports::http;
use uckb_jsonrpc_core::client::HttpClient as RawHttpClient;
use url::Url;

use crate::{error::Result, runtime::Runtime};

mod methods;

pub(super) struct HttpClient {
    client: RawHttpClient,
}

impl HttpClient {
    pub(super) fn new(rt: Runtime, url: &Url) -> Result<Self> {
        log::trace!("initialize a http client to connect {}", url);
        let fut_conn = http::connect::<RawHttpClient>(url.as_str());
        let client = rt.block_on_01(fut_conn)?;
        Ok(Self { client })
    }

    pub(super) fn client(&self) -> RawHttpClient {
        self.client.clone()
    }
}
