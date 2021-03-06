//  lib.rs
//  punch-clock
//
//  Created by Søren Mortensen <soren@neros.dev> on 2019-12-26.
//  Copyright (c) 2019 Søren Mortensen.
//
//  Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
//  http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
//  http://opensource.org/licenses/MIT>, at your option. This file may not be
//  copied, modified, or distributed except according to those terms.

//! The `punch-clock` crate is a lightweight tool for tracking time.
//!
//! This library exposes an API for performing all the same tasks as through the command-line
//! interface (e.g. punching in or out, checking time tracking status, counting totals).

mod event;
mod period;
pub mod sheet;

pub use event::Event;
pub use period::Period;
pub use sheet::Sheet;
