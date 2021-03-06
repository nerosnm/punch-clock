//  event.rs
//  punch-clock
//
//  Created by Søren Mortensen <soren@neros.dev> on 2020-02-29.
//  Copyright (c) 2020 Søren Mortensen.
//
//  Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
//  http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
//  http://opensource.org/licenses/MIT>, at your option. This file may not be
//  copied, modified, or distributed except according to those terms.

use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

/// Represents a (possibly ongoing) period of time tracking, with its associated metadata.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Event {
    /// The start of a time-tracking period.
    pub start: DateTime<Utc>,
    /// The end of a time-tracking period.
    pub stop: Option<DateTime<Utc>>,
}

impl Event {
    /// Create a new event starting at the given time.
    pub fn new(start: DateTime<Utc>) -> Self {
        Event { start, stop: None }
    }
}
