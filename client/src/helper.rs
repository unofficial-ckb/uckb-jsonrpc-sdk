// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

macro_rules! option_to_error {
    ($opt:ident, $default:expr) => {{
        if let Some(inner) = $opt {
            Ok(inner)
        } else {
            Err(())
        }
        .or_else(|_| $default)
    }};
}

macro_rules! option_to_future {
    ($opt:ident, $default:expr) => {{
        let fut = $default;
        if let Some(inner) = $opt {
            future::ok(inner)
        } else {
            future::err(())
        }
        .or_else(|_| fut)
    }};
}
