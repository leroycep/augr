use crate::{
    repository::event::{Error as EventError, PatchedEvent},
    EventRef, Timesheet,
};
use chrono::{DateTime, Utc};
use snafu::Snafu;
use std::collections::BTreeMap;

/// This representation of a timesheet is an intermediate form that allows
/// an event to have multiple starts
#[derive(Clone, Debug)]
pub struct PatchedTimesheet {
    pub events: BTreeMap<EventRef, PatchedEvent>,
}

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not flatten event {}: {}", event, source))]
    FlattenEventError { source: EventError, event: EventRef },

    #[snafu(display(
        "Two events have the same start time (events \"{}\" and \"{}\")",
        event_a,
        event_b
    ))]
    DuplicateEventTime {
        event_a: EventRef,
        event_b: EventRef,
    },
}

impl PatchedTimesheet {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }

    pub fn flatten(&self) -> Result<Timesheet, Vec<Error>> {
        let mut timesheet = Timesheet::new();
        let mut errors = Vec::new();
        let mut event_datetimes_to_refs: BTreeMap<DateTime<Utc>, EventRef> = BTreeMap::new();
        for (event_ref, patched_event) in self.events.iter() {
            match patched_event.flatten() {
                Ok(event) => {
                    if let Some(_event_a_tags) = timesheet.insert_event(event.clone()) {
                        errors.push(Error::DuplicateEventTime {
                            event_a: event_datetimes_to_refs[event.start()].clone(),
                            event_b: event_ref.clone(),
                        });
                    }
                    event_datetimes_to_refs.insert(event.start().clone(), event_ref.clone());
                }
                Err(source) => {
                    errors.push(Error::FlattenEventError {
                        source,
                        event: event_ref.clone(),
                    });
                }
            }
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(timesheet)
        }
    }
}
