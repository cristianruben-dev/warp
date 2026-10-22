//! Module containing utilities to query the currently running antivirus / EDR software on the
//! user's machine.

mod telemetry;

use warpui::{Entity, SingletonEntity};

/// Singleton model that reports the currently running antivirus software.
#[derive(Debug, Clone)]
pub struct AntivirusInfo(Option<String>);

impl AntivirusInfo {
    pub fn new() -> Self {
        Self(None)
    }

    /// Returns the currently running antivirus software if any.
    /// If called before the antivirus is computed (i.e. before
    /// [`AntivirusInfoEvent::ScannedComplete`] is emitted), this function returns [`None`].
    pub fn get(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

pub enum AntivirusInfoEvent {
    #[allow(dead_code)]
    ScannedComplete,
}

impl Entity for AntivirusInfo {
    type Event = AntivirusInfoEvent;
}

impl SingletonEntity for AntivirusInfo {}
