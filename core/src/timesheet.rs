use crate::Tag;
use chrono::{DateTime, Utc, Duration};
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

#[derive(Clone, Debug)]
pub struct Segment {
    pub start_time: DateTime<Utc>,
    pub tags: BTreeSet<Tag>,
    pub duration: Duration,
    pub end_time: DateTime<Utc>,
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

    pub fn events(&self) -> &BTreeMap<DateTime<Utc>, BTreeSet<Tag>> {
        &self.events
    }

    pub fn segments(&self) -> Vec<Segment> {
        let now = Utc::now();
        let end_cap_arr = [now];
        self.events
            .iter()
            .zip(self.events.keys().skip(1).chain(end_cap_arr.iter()))
            .map(|(t, end_time)| {
                let duration = end_time.signed_duration_since(*t.0);
                Segment {
                    start_time: t.0.clone(),
                    tags: (*t.1).clone(),
                    duration,
                    end_time: end_time.clone(),
                }
            })
            .collect()
    }

    pub fn tags_at_time<'ts>(&'ts self, datetime: &DateTime<Utc>) -> Option<&'ts BTreeSet<Tag>> {
        self.events
            .range::<DateTime<_>, _>(..datetime)
            .map(|(_time, tags)| tags)
            .last()
    }

}
