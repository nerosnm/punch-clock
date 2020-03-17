//  sheet.rs
//  punch-clock
//
//  Created by Søren Mortensen <soren@neros.dev> on 2020-03-01.
//  Copyright (c) 2020 Søren Mortensen.
//
//  Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
//  http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
//  http://opensource.org/licenses/MIT>, at your option. This file may not be
//  copied, modified, or distributed except according to those terms.

//! Working with recorded timesheets (lists of events).

use chrono::{DateTime, Duration, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{
    fs::File,
    io::{Read, Write},
};

use crate::Event;

/// List of events, together comprising a log of work from which totals can be calculated for
/// various periods of time.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Sheet {
    pub events: Vec<Event>,
}

impl Sheet {
    /// Attempt to load a sheet from the file at the default location.
    ///
    /// The default location is determined using the [directories][directories] crate by platform
    /// as follows:
    ///
    /// + Linux: `$XDG_CONFIG_HOME/punchclock/sheet.json`
    /// + macOS: `$HOME/Library/Application Support/dev.neros.PunchClock/sheet.json`
    /// + Windows: `%APPDATA%\Local\Neros\PunchClock\sheet.json`
    ///
    /// [directories]: https://crates.io/crates/directories
    pub fn load_default() -> Result<Sheet, SheetError> {
        let project_dirs =
            ProjectDirs::from("dev", "neros", "PunchClock").ok_or(SheetError::FindSheet)?;
        let data_dir = project_dirs.data_dir().to_owned();

        let mut sheet_path = data_dir.clone();
        sheet_path.push("sheet.json");

        let mut sheet_json = String::new();

        {
            let mut sheet_file = File::open(&sheet_path).map_err(SheetError::OpenSheet)?;

            sheet_file
                .read_to_string(&mut sheet_json)
                .map_err(SheetError::ReadSheet)?;
        }

        if sheet_json.is_empty() {
            Ok(Sheet::default())
        } else {
            serde_json::from_str(&sheet_json).map_err(SheetError::ParseSheet)
        }
    }

    /// Attempt to write a sheet to the file at the default location.
    ///
    /// The default location is determined using the [directories][directories] crate by platform
    /// as follows:
    ///
    /// + Linux: `$XDG_CONFIG_HOME/punchclock/sheet.json`
    /// + macOS: `$HOME/Library/Application Support/dev.neros.PunchClock/sheet.json`
    /// + Windows: `%APPDATA%\Local\Neros\PunchClock\sheet.json`
    ///
    /// [directories]: https://crates.io/crates/directories
    pub fn write_default(&self) -> Result<(), SheetError> {
        let new_sheet_json = serde_json::to_string(self).unwrap();

        let project_dirs =
            ProjectDirs::from("dev", "neros", "PunchClock").ok_or(SheetError::FindSheet)?;

        let mut sheet_path = project_dirs.data_dir().to_owned();
        sheet_path.push("sheet.json");

        match File::create(&sheet_path) {
            Ok(mut sheet_file) => {
                write!(&mut sheet_file, "{}", new_sheet_json).map_err(SheetError::WriteSheet)
            }
            Err(e) => Err(SheetError::WriteSheet(e)),
        }
    }

    /// Record a punch-in (start of a time-tracking period) at the current time.
    pub fn punch_in(&mut self) -> Result<DateTime<Utc>, SheetError> {
        self.punch_in_at(Utc::now())
    }

    /// Record a punch-in (start of a time-tracking period) at the given time.
    pub fn punch_in_at(&mut self, time: DateTime<Utc>) -> Result<DateTime<Utc>, SheetError> {
        match self.events.last() {
            Some(Event { stop: Some(_), .. }) | None => {
                let event = Event::new(time);
                self.events.push(event);
                Ok(time)
            }
            Some(Event {
                start: start_time, ..
            }) => Err(SheetError::PunchedIn(*start_time)),
        }
    }

    /// Record a punch-out (end of a time-tracking period) at the current time.
    pub fn punch_out(&mut self) -> Result<DateTime<Utc>, SheetError> {
        self.punch_out_at(Utc::now())
    }

    /// Record a punch-out (end of a time-tracking period) at the given time.
    pub fn punch_out_at(&mut self, time: DateTime<Utc>) -> Result<DateTime<Utc>, SheetError> {
        match self.events.last_mut() {
            Some(ref mut event @ Event { stop: None, .. }) => {
                event.stop = Some(time);
                Ok(time)
            }
            Some(Event {
                stop: Some(stop_time),
                ..
            }) => Err(SheetError::PunchedOut(*stop_time)),
            None => Err(SheetError::NoPunches),
        }
    }

    /// Get the current status of time-tracking, including the time at which the status last
    /// changed.
    pub fn status(&self) -> SheetStatus {
        match self.events.last() {
            Some(Event {
                stop: Some(stop), ..
            }) => SheetStatus::PunchedOut(*stop),
            Some(Event { start, .. }) => SheetStatus::PunchedIn(*start),
            None => SheetStatus::Empty,
        }
    }

    /// Count the amount of time for which there was recorded work between the two given instants,
    /// including an ongoing time-tracking period if there is one.
    pub fn count_range(&self, begin: DateTime<Utc>, end: DateTime<Utc>) -> Duration {
        self.events
            .iter()
            .map(|e| (e.start, e.stop.unwrap_or(Utc::now())))
            .filter(|(start, stop)| {
                let entirely_before = start < &begin && stop < &begin;
                let entirely_after = start > &end && stop > &end;

                !(entirely_before || entirely_after)
            })
            .map(|(start, stop)| {
                let real_begin = std::cmp::max(begin, start);
                let real_end = std::cmp::min(end, stop);

                real_end - real_begin
            })
            .fold(Duration::zero(), |acc, next| acc + next)
    }
}

impl Default for Sheet {
    fn default() -> Self {
        Sheet { events: vec![] }
    }
}

/// Whether or not time is currently being tracked.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SheetStatus {
    /// Time is currently being tracked, and has been since the given instant.
    PunchedIn(DateTime<Utc>),
    /// Time is not currently being tracked, as of the given instant.
    PunchedOut(DateTime<Utc>),
    /// No time has ever been tracked.
    Empty,
}

/// Errors arising through the use of [`Sheet`][sheet].
///
/// [sheet]: ./struct.Sheet.html
#[derive(Error, Debug)]
pub enum SheetError {
    #[error("already punched in at {0}")]
    PunchedIn(DateTime<Utc>),
    #[error("not punched in, last punched out at {0}")]
    PunchedOut(DateTime<Utc>),
    #[error("not punched in, no punch-ins recorded")]
    NoPunches,
    #[error("unable to find sheet file")]
    FindSheet,
    #[error("unable to open sheet file")]
    OpenSheet(#[source] std::io::Error),
    #[error("unable to read sheet file")]
    ReadSheet(#[source] std::io::Error),
    #[error("unable to parse sheet")]
    ParseSheet(#[source] serde_json::Error),
    #[error("unable to write sheet to file")]
    WriteSheet(#[source] std::io::Error),
}
