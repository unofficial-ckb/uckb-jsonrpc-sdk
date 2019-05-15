// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use failure::Fail;

use bincode::Error as BincodeError;
use rkv::StoreError;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "storage error: {:?}", _0)]
    Store(StoreError),

    #[fail(display = "serialization error: {:?}", _0)]
    Serde(BincodeError),

    #[fail(display = "corruption error: {}", _0)]
    Corruption(String),

    #[fail(display = "custom error: {}", _0)]
    Custom(String),
}

impl Error {
    pub fn corruption(msg: &str) -> Self {
        Error::Corruption(msg.to_owned())
    }

    pub fn custom(msg: &str) -> Self {
        Error::Custom(msg.to_owned())
    }
}

macro_rules! define_error {
    ($err:ident, $from:ident) => {
        impl From<$from> for Error {
            fn from(err: $from) -> Self {
                Error::$err(err)
            }
        }
    };
}

define_error!(Store, StoreError);
define_error!(Serde, BincodeError);
