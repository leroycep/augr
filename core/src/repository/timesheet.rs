use crate::{
    repository::event::{Error as EventError, PatchedEvent},
    EventRef, Patch, PatchRef, Timesheet,
};
use chrono::{DateTime, Utc};
use snafu::Snafu;
use std::collections::BTreeMap;

/// This representation of a timesheet is an intermediate form that allows
/// an event to have multiple starts
#[derive(Default, Clone, Debug)]
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

    #[snafu(display("Unknown event {} in patch {}", event, patch))]
    UnknownEvent { patch: PatchRef, event: EventRef },

    #[snafu(display("Two events were created with the same id {}", id))]
    DuplicateEventId { id: EventRef },
}

impl PatchedTimesheet {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }

    pub fn apply_patch(&mut self, patch: &Patch) -> Result<(), Vec<Error>> {
        // Verify patch. From this point on, we should have no errors, and `expect("valid patch")` indicates that
        if let Err(errors) = self.verify_patch(patch) {
            return Err(errors);
        }
        let patch_ref = patch.patch_ref();

        for start_added in patch.add_start.iter() {
            let event = self
                .events
                .get_mut(&start_added.event)
                .expect("valid patch");
            event.add_start(*patch_ref, start_added.time);

            // Update metadata
            for parent in start_added.parents() {
                event.remove_patch_from_latest(&parent);
            }
            event.add_patch_to_latest(patch_ref.clone());
        }
        for start_removed in patch.remove_start.iter() {
            let event = self
                .events
                .get_mut(&start_removed.event)
                .expect("valid patch");
            event.remove_start(start_removed.patch, start_removed.time);

            // Update metadata
            event.remove_patch_from_latest(&start_removed.patch);
            for parent in start_removed.parents() {
                event.remove_patch_from_latest(&parent);
            }
            event.add_patch_to_latest(patch_ref.clone());
        }

        for tag_added in patch.add_tag.iter() {
            let event = self.events.get_mut(&tag_added.event).expect("valid patch");
            event.add_tag(patch_ref.clone(), tag_added.tag.clone());

            // Update metadata
            for parent in tag_added.parents() {
                event.remove_patch_from_latest(&parent);
            }
            event.add_patch_to_latest(patch_ref.clone());
        }
        for tag_removed in patch.remove_tag.iter() {
            let event = self
                .events
                .get_mut(&tag_removed.event)
                .expect("valid patch");
            event.remove_tag(tag_removed.patch, tag_removed.tag.clone());

            // Update metadata
            event.remove_patch_from_latest(&tag_removed.patch);
            for parent in tag_removed.parents() {
                event.remove_patch_from_latest(&parent);
            }
            event.add_patch_to_latest(patch_ref.clone());
        }

        for new_event in patch.create_event.iter() {
            let mut event = PatchedEvent::new();
            event.add_start(patch_ref.clone(), new_event.start);
            for tag in new_event.tags.iter().cloned() {
                event.add_tag(patch_ref.clone(), tag);
            }

            // Update metadata
            event.add_patch_to_latest(patch_ref.clone());

            let prev_entry = self.events.insert(new_event.event.clone(), event);
            assert!(prev_entry.is_none());
        }

        Ok(())
    }

    fn verify_patch(&self, patch: &Patch) -> Result<(), Vec<Error>> {
        let mut errors = Vec::new();
        let patch_ref = patch.patch_ref();

        // Pull out every event reference into an iterator
        let add_start_events = patch.add_start.iter().map(|s| s.event.as_str());
        let remove_start_events = patch.remove_start.iter().map(|s| s.event.as_str());
        let add_tag_events = patch.add_tag.iter().map(|s| s.event.as_str());
        let remove_tag_events = patch.remove_tag.iter().map(|s| s.event.as_str());
        let events = add_start_events
            .chain(remove_start_events)
            .chain(add_tag_events)
            .chain(remove_tag_events);

        // Make sure that all EventReferences are valid
        for event in events {
            if self.events.get(event).is_none() {
                errors.push(Error::UnknownEvent {
                    patch: *patch_ref,
                    event: event.to_string(),
                });
                continue;
            };
        }

        // Make sure that new events don't have the same ID as some other event
        for new_event in patch.create_event.iter() {
            if self.events.get(&new_event.event).is_some() {
                errors.push(Error::DuplicateEventId {
                    id: new_event.event.clone(),
                });
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn flatten(&self) -> Result<Timesheet<'_>, Vec<Error>> {
        let mut timesheet = Timesheet::new(&self);
        let mut errors = Vec::new();
        let mut event_datetimes_to_refs: BTreeMap<DateTime<Utc>, EventRef> = BTreeMap::new();
        for (event_ref, patched_event) in self.events.iter() {
            match patched_event.flatten() {
                Ok(event) => {
                    if let Some(_event_a_tags) =
                        timesheet.event_at_time(event.start().clone(), event_ref.clone())
                    {
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

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(timesheet)
        }
    }
}
