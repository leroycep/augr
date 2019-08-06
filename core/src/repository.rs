use crate::{Event, EventRef, PatchRef, Store, Tag, Timesheet};
use chrono::{DateTime, Utc};
use snafu::{ensure, Snafu};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Snafu)]
pub enum Error<IE>
where
    IE: std::error::Error + 'static,
{
    #[snafu(display("Unable to load patch {}: {}", patch, source))]
    PatchNotFound { source: IE, patch: PatchRef },

    #[snafu(display("Event {}, referenced from patch {}, was not found", event, patch))]
    EventNotFound { patch: PatchRef, event: EventRef },
}

pub struct Repository<S: Store> {
    store: S,
    device_id: String,
}

impl<S: Store> Repository<S> {
    pub fn from_store(store: S, device_id: String) -> Self {
        Self { store, device_id }
    }

    pub fn get_current_timesheet(&self) -> Result<PatchedTimesheet, Vec<Error<S::Error>>> {
        let mut timesheet = PatchedTimesheet::new();
        let mut errors = Vec::new();

        let meta = self.store.get_device_meta(&self.device_id).unwrap();
        let mut patches_to_load: VecDeque<_> = meta.patches().cloned().collect();
        let mut patches_loaded = BTreeSet::new();
        while let Some(patch_ref) = patches_to_load.pop_front() {
            let patch = match self.store.get_patch(&patch_ref) {
                Ok(p) => p,
                Err(source) => {
                    errors.push(Error::PatchNotFound {
                        source,
                        patch: patch_ref.clone(),
                    });
                    continue;
                }
            };

            // Check that all of the patches parent patches have been loaded
            let mut all_parents_loaded = true;
            for parent_patch_ref in patch.parents() {
                if !patches_loaded.contains(&parent_patch_ref) {
                    all_parents_loaded = false;
                    patches_to_load.push_back(parent_patch_ref.clone());
                }
            }
            if !all_parents_loaded {
                // If not all parents are loaded, put the current patch at the back and continue
                patches_to_load.push_back(patch_ref);
                continue;
            }

            // Mark patch as loaded
            patches_loaded.insert(patch_ref.clone());

            for start_added in patch.add_start.iter() {
                let event = match timesheet.events.get_mut(&start_added.event) {
                    Some(event) => event,
                    None => {
                        errors.push(Error::EventNotFound {
                            patch: patch_ref.clone(),
                            event: start_added.event.clone(),
                        });
                        continue;
                    }
                };
                event.add_start(patch_ref.clone(), start_added.time.clone())
            }
            for start_removed in patch.remove_start.iter() {
                let event = timesheet
                    .events
                    .get_mut(&start_removed.event)
                    .expect("no event for remove-start");
                event.remove_start(start_removed.patch.clone(), start_removed.time.clone())
            }

            for tag_added in patch.add_tag.iter() {
                let event = timesheet
                    .events
                    .get_mut(&tag_added.event)
                    .expect("no event for add-tag");
                event.add_tag(patch_ref.clone(), tag_added.tag.clone())
            }
            for tag_removed in patch.remove_tag.iter() {
                let event = timesheet
                    .events
                    .get_mut(&tag_removed.event)
                    .expect("no event for remove-tag");
                event.remove_tag(tag_removed.patch.clone(), tag_removed.tag.clone())
            }

            for new_event in patch.create_event.iter() {
                let mut event = PatchedEvent::new();
                event.add_start(patch_ref.clone(), new_event.start);
                for tag in new_event.tags.iter().cloned() {
                    event.add_tag(patch_ref.clone(), tag);
                }
                timesheet.events.insert(new_event.event.clone(), event);
            }
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(timesheet)
        }
    }
}

/// This representation of a timesheet is an intermediate form that allows
/// an event to have multiple starts
pub struct PatchedTimesheet {
    events: BTreeMap<EventRef, PatchedEvent>,
}

pub struct PatchedEvent {
    starts_added: BTreeSet<(PatchRef, DateTime<Utc>)>,
    starts_removed: BTreeSet<(PatchRef, DateTime<Utc>)>,
    tags_added: BTreeSet<(PatchRef, String)>,
    tags_removed: BTreeSet<(PatchRef, String)>,
}

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum TimesheetError {
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
    fn new() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }

    pub fn flatten(&self) -> Result<Timesheet, Vec<TimesheetError>> {
        let mut timesheet = Timesheet::new();
        let mut errors = Vec::new();
        let mut event_datetimes_to_refs: BTreeMap<DateTime<Utc>, EventRef> = BTreeMap::new();
        for (event_ref, patched_event) in self.events.iter() {
            match patched_event.flatten() {
                Ok(event) => {
                    if let Some(_event_a_tags) = timesheet.insert_event(event.clone()) {
                        errors.push(TimesheetError::DuplicateEventTime {
                            event_a: event_datetimes_to_refs[event.start()].clone(),
                            event_b: event_ref.clone(),
                        });
                    }
                    event_datetimes_to_refs.insert(event.start().clone(), event_ref.clone());
                }
                Err(source) => {
                    errors.push(TimesheetError::FlattenEventError {
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

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum EventError {
    #[snafu(display("Event has multiple start times"))]
    MultipleStartTimes,

    #[snafu(display("Event has no start times"))]
    NoStartTimes,
}

impl PatchedEvent {
    fn new() -> Self {
        Self {
            starts_added: BTreeSet::new(),
            starts_removed: BTreeSet::new(),
            tags_added: BTreeSet::new(),
            tags_removed: BTreeSet::new(),
        }
    }

    fn add_start(&mut self, patch: PatchRef, datetime: DateTime<Utc>) {
        self.starts_added.insert((patch, datetime));
    }

    fn remove_start(&mut self, patch: PatchRef, datetime: DateTime<Utc>) {
        self.starts_removed.insert((patch, datetime));
    }

    pub fn starts(&self) -> BTreeSet<(PatchRef, DateTime<Utc>)> {
        self.starts_added
            .difference(&self.starts_removed)
            .cloned()
            .collect()
    }

    fn add_tag(&mut self, patch: PatchRef, tag: Tag) {
        self.tags_added.insert((patch, tag));
    }

    fn remove_tag(&mut self, patch: PatchRef, tag: Tag) {
        self.tags_removed.insert((patch, tag));
    }

    pub fn tags(&self) -> BTreeSet<(PatchRef, Tag)> {
        self.tags_added
            .difference(&self.tags_removed)
            .cloned()
            .collect()
    }

    pub fn flatten(&self) -> Result<Event, EventError> {
        let starts = self.starts();
        ensure!(starts.len() < 2, MultipleStartTimes);
        ensure!(starts.len() > 0, NoStartTimes);
        let start = starts
            .iter()
            .map(|patch_and_dt| patch_and_dt.1)
            .next()
            .expect("should be exactly one start");
        let tags = self
            .tags_added
            .difference(&self.tags_removed)
            .cloned()
            .map(|patch_and_tag| patch_and_tag.1)
            .collect();
        Ok(Event::new(start, tags))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn remove_start_from_event() {
        let dt0 = Utc.ymd(2019, 07, 23).and_hms(12, 0, 0);
        let dt1 = Utc.ymd(2019, 07, 23).and_hms(12, 30, 0);

        let mut event = PatchedEvent::new();
        event.add_start("a".into(), dt0);
        event.add_start("a".into(), dt1);
        event.remove_start("a".into(), dt0);

        assert_eq!(
            event.starts(),
            [("a".into(), dt1)].into_iter().cloned().collect()
        );
    }

    #[test]
    fn remove_tag_from_event() {
        let mut event = PatchedEvent::new();
        event.add_tag("a".into(), "hello".into());
        event.add_tag("a".into(), "world".into());
        event.remove_tag("a".into(), "world".into());

        assert_eq!(
            event.tags(),
            [("a".into(), "hello".into())]
                .into_iter()
                .cloned()
                .collect()
        );
    }
}
