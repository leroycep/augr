use crate::{repository::timesheet::PatchedTimesheet, EventRef, Tag};
use chrono::{DateTime, Duration, Utc};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct Event {
    start: DateTime<Utc>,
    tags: BTreeSet<Tag>,
}

#[derive(Clone, Debug)]
pub struct Timesheet<'cl> {
    patched_timesheet: &'cl PatchedTimesheet,
    event_starts: BTreeMap<DateTime<Utc>, EventRef>,
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub event_ref: EventRef,
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

impl<'a, 'b> PartialEq<Timesheet<'b>> for Timesheet<'a> {
    fn eq(&self, other: &Timesheet) -> bool {
        self.events().eq(&other.events())
    }
}

impl Eq for Timesheet<'_> {}

impl PartialEq<BTreeMap<DateTime<Utc>, BTreeSet<Tag>>> for Timesheet<'_> {
    fn eq(&self, other: &BTreeMap<DateTime<Utc>, BTreeSet<Tag>>) -> bool {
        self.events().eq(other)
    }
}

impl<'cl> Timesheet<'cl> {
    pub fn new(patched_timesheet: &'cl PatchedTimesheet) -> Self {
        Self {
            patched_timesheet,
            event_starts: BTreeMap::new(),
        }
    }

    pub fn get_patched_timesheet(&'cl self) -> &'cl PatchedTimesheet {
        &self.patched_timesheet
    }

    pub fn event_at_time(&mut self, start: DateTime<Utc>, event_ref: EventRef) -> Option<EventRef> {
        match self.event_starts.insert(start, event_ref) {
            None => None,
            Some(previous_event_ref) => Some(previous_event_ref),
        }
    }

    pub fn events(&self) -> BTreeMap<DateTime<Utc>, BTreeSet<Tag>> {
        self.event_starts
            .iter()
            .map(|(start, event_ref)| {
                let tags = self.patched_timesheet.events[event_ref]
                    .tags()
                    .into_iter()
                    .map(|(_patch_ref, tag)| tag)
                    .collect();
                (start.clone(), tags)
            })
            .collect()
    }

    pub fn segments(&self) -> Vec<Segment> {
        let now = Utc::now();
        let end_cap_arr = [now];
        self.event_starts
            .iter()
            .zip(self.event_starts.keys().skip(1).chain(end_cap_arr.iter()))
            .map(|((start_time, event_ref), end_time)| {
                let event = &self.patched_timesheet.events[event_ref];
                let duration = end_time.signed_duration_since(*start_time);
                Segment {
                    event_ref: event_ref.clone(),
                    start_time: start_time.clone(),
                    tags: event.tags().into_iter().map(|(_ref, tag)| tag).collect(),
                    duration,
                    end_time: end_time.clone(),
                }
            })
            .collect()
    }

    pub fn tags_at_time<'ts>(&'ts self, datetime: &DateTime<Utc>) -> Option<BTreeSet<Tag>> {
        self.event_starts
            .range::<DateTime<_>, _>(..datetime)
            .last()
            .map(|(_time, event_ref)| {
                self.patched_timesheet.events[event_ref]
                    .tags()
                    .into_iter()
                    .map(|(_patch_ref, tag)| tag)
                    .collect()
            })
    }
}
