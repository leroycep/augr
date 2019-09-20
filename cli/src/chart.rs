use crate::format_duration;
use augr_core::{timesheet::Segment, Tag, Timesheet};
use chrono::{offset::TimeZone, DateTime, Duration, Local, NaiveDate, Utc};
use std::collections::BTreeSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "chart")]
pub struct Cmd {
    /// A list of tags to filter against
    tags: Vec<String>,

    /// The date to start charting from. Defaults to 7 days ago.
    #[structopt(long = "start")]
    start: Option<NaiveDate>,

    /// The date to stop charting at. Defaults to today.
    #[structopt(long = "end")]
    end: Option<NaiveDate>,
}

impl Cmd {
    pub fn exec(&self, timesheet: &Timesheet) {
        let tags: BTreeSet<Tag> = self.tags.iter().cloned().map(Tag::from).collect();

        let now = chrono::Local::now();
        let end_date = match self.end {
            Some(naive_date) => Local.from_local_date(&naive_date).unwrap(),
            None => chrono::Local::today(),
        };
        let start_date = match self.start {
            Some(naive_date) => Local.from_local_date(&naive_date).unwrap(),
            None => end_date - chrono::Duration::days(6),
        };

        let mut cur_date = start_date;
        let mut total_time = Duration::seconds(0);

        print!("Day ");
        for hour in 0..24 {
            print!("{: <3}", hour);
        }
        println!();

        while cur_date <= end_date {
            print!("{} ", cur_date.format("%a"));
            for section in 0..(24 * 3) {
                let hour = section / 3;
                let minutes = (section % 3) * 20;
                let cur_datetime = cur_date.and_hms(hour, minutes, 0);
                let cur_tags = timesheet.tags_at_time(&cur_datetime.with_timezone(&Utc));
                let matches = cur_tags
                    .map(|x| tags.is_subset(&x) && !x.is_empty())
                    .unwrap_or(false);

                // Avoid highlighting the entire day
                let in_past = cur_datetime <= now;

                if matches && in_past {
                    print!("█");
                } else {
                    print!(" ");
                }
            }
            let time_for_day = total_duration_in_range(
                timesheet,
                cur_date.and_hms(0, 0, 0).with_timezone(&Utc),
                cur_date.succ().and_hms(0, 0, 0).with_timezone(&Utc),
                &tags,
            );
            println!(" {}", format_duration(time_for_day));
            total_time = total_time + time_for_day;
            cur_date = cur_date + chrono::Duration::days(1);
        }

        println!("\ntotal time: {}", format_duration(total_time));
    }
}

fn total_duration_in_range(
    timesheet: &Timesheet,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    tags: &BTreeSet<Tag>,
) -> Duration {
    assert!(start < end);
    timesheet
        .segments()
        .into_iter()
        .filter(|s| s.end_time >= start)
        .filter(|s| s.start_time <= end)
        .filter(|s| !s.tags.is_empty() && tags.is_subset(&s.tags))
        .filter_map(|s| segment_duration_in_range(&s, start, end))
        .fold(Duration::seconds(0), |acc, d| acc + d)
}

/// Make sure the start and end are no earlier or later the given start and end
fn segment_duration_in_range(
    segment: &Segment,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Option<Duration> {
    assert!(start < end);
    if segment.start_time > end || segment.end_time < start {
        None
    } else {
        let start_time = segment.start_time.max(start);
        let end_time = segment.end_time.min(end);
        Some(end_time.signed_duration_since(start_time))
    }
}
