use crate::Tag;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct Event {
    start: DateTime<Utc>,
    tags: BTreeSet<Tag>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Timesheet {
    events: BTreeMap<DateTime<Utc>, BTreeSet<Tag>>,
}

impl Event {
    pub fn new(start: DateTime<Utc>, tags: BTreeSet<Tag>) -> Self {
        Self { start, tags }
    }

    pub fn start(&self) -> &DateTime<Utc> {
        &self.start
    }

    pub fn tags(&self) -> &BTreeSet<Tag> {
        &self.tags
    }
}

impl Timesheet {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }

    pub fn insert_event(&mut self, event: Event) -> Option<Event> {
        match self.events.insert(event.start.clone(), event.tags) {
            None => None,
            Some(previous_event_tags) => Some(Event {
                start: event.start,
                tags: previous_event_tags,
            }),
        }
    }
}
