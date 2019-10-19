// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub extern crate uckb_jsonrpc_interfaces as interfaces;
pub extern crate url;

pub mod sdk {
    pub extern crate jsonrpc_sdk_client as client;
    pub extern crate jsonrpc_sdk_prelude as prelude;
}

pub mod client;
