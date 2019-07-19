use crate::timesheet::{Segment, Tag};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};

pub trait DataBase {
    fn transitions(&self) -> BTreeMap<&DateTime<Utc>, &BTreeSet<Tag>>;
    fn insert_transition(&mut self, datetime: DateTime<Utc>, tags: BTreeSet<Tag>);

    fn tags_at_time<'ts>(&'ts self, datetime: &DateTime<Utc>) -> Option<&'ts BTreeSet<Tag>> {
        self.transitions()
            .range::<DateTime<_>, _>(..datetime)
            .map(|(_time, tags)| *tags)
            .last()
    }

    fn segments(&self) -> Vec<Segment> {
        let now = Utc::now();
        let end_cap_arr = [&now];
        let transitions = self.transitions();
        transitions
            .iter()
            .zip(transitions.keys().skip(1).chain(end_cap_arr.iter()))
            .map(|(t, end_time)| {
                let duration = end_time.signed_duration_since(**t.0);
                Segment {
                    start_time: *t.0.clone(),
                    tags: (*t.1).clone(),
                    duration,
                    end_time: *end_time.clone(),
                }
            })
            .collect()
    }
}
