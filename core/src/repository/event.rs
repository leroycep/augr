use crate::{Event, PatchRef, Tag};
use chrono::{DateTime, Utc};
use snafu::{ensure, Snafu};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct PatchedEvent {
    starts_added: BTreeSet<(PatchRef, DateTime<Utc>)>,
    starts_removed: BTreeSet<(PatchRef, DateTime<Utc>)>,
    tags_added: BTreeSet<(PatchRef, String)>,
    tags_removed: BTreeSet<(PatchRef, String)>,
}

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum Error {
    #[snafu(display("Event has multiple start times"))]
    MultipleStartTimes,

    #[snafu(display("Event has no start times"))]
    NoStartTimes,
}

impl PatchedEvent {
    pub fn new() -> Self {
        Self {
            starts_added: BTreeSet::new(),
            starts_removed: BTreeSet::new(),
            tags_added: BTreeSet::new(),
            tags_removed: BTreeSet::new(),
        }
    }

    pub fn add_start(&mut self, patch: PatchRef, datetime: DateTime<Utc>) {
        self.starts_added.insert((patch, datetime));
    }

    pub fn remove_start(&mut self, patch: PatchRef, datetime: DateTime<Utc>) {
        self.starts_removed.insert((patch, datetime));
    }

    pub fn starts(&self) -> BTreeSet<(PatchRef, DateTime<Utc>)> {
        self.starts_added
            .difference(&self.starts_removed)
            .cloned()
            .collect()
    }

    pub fn add_tag(&mut self, patch: PatchRef, tag: Tag) {
        self.tags_added.insert((patch, tag));
    }

    pub fn remove_tag(&mut self, patch: PatchRef, tag: Tag) {
        self.tags_removed.insert((patch, tag));
    }

    pub fn tags(&self) -> BTreeSet<(PatchRef, Tag)> {
        self.tags_added
            .difference(&self.tags_removed)
            .cloned()
            .collect()
    }

    pub fn flatten(&self) -> Result<Event, Error> {
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
