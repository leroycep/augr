use crate::{format_duration, time_input::parse_default_local};
use augr_core::{Tag, Timesheet};
use chrono::{DateTime, Local};
use std::collections::BTreeSet;
use structopt::StructOpt;

#[derive(StructOpt, Default, Debug)]
pub struct SummaryCmd {
    /// A list of tags to filter against
    tags: Vec<String>,

    #[structopt(long = "show-ends")]
    show_ends: bool,

    /// Show event references as tags in output
    #[structopt(long = "refs")]
    show_refs: bool,

    #[structopt(long = "start", parse(try_from_os_str = parse_default_local))]
    start: Option<DateTime<Local>>,

    #[structopt(long = "end", parse(try_from_os_str = parse_default_local))]
    end: Option<DateTime<Local>>,
}

impl SummaryCmd {
    #[cfg_attr(feature = "flame_it", flame)]
    pub fn exec(&self, timesheet: &Timesheet) {
        let tags: BTreeSet<Tag> = self.tags.iter().cloned().collect();

        let start = self.start.unwrap_or_else(default_start);
        let end = self.end.unwrap_or_else(default_end);
        let segments = timesheet
            .segments()
            .into_iter()
            .filter(|s| s.start_time.with_timezone(&Local) >= start)
            .filter(|s| s.start_time.with_timezone(&Local) <= end)
            .filter(|s| s.tags.is_superset(&tags));

        let mut total_duration = chrono::Duration::seconds(0);
        let mut current_date = None;

        if !self.show_ends {
            println!("Date  Start Duration Total     Tags");
            println!(
                "――――― ――――― ―――――――― ――――――――  ――――――――"
            );
        } else {
            println!("Date  Start End   Duration Total     Tags");
            println!(
                "――――― ――――― ――――― ―――――――― ――――――――  ――――――――"
            );
        }
        for segment in segments {
            let seg_datetime = segment.start_time.with_timezone(&chrono::Local);
            let seg_end_datetime = segment.end_time.with_timezone(&chrono::Local);
            let seg_date = seg_datetime.date();
            let date_str = if current_date != Some(seg_date) {
                current_date = Some(seg_date);
                seg_date.format("%m/%d").to_string()
            } else {
                String::from("     ")
            };
            let start_time = seg_datetime.format("%H:%M");
            let end_time = seg_end_datetime.format("%H:%M");

            let reference = if self.show_refs {
                Some(segment.event_ref.as_str())
            } else {
                None
            };

            let tags_str = segment
                .tags
                .iter()
                .map(|s| &**s)
                .chain(reference)
                .collect::<Vec<&str>>()
                .join(" ");

            total_duration = total_duration + segment.duration;

            let duration_str = format_duration(segment.duration);
            let total_duration_str = format_duration(total_duration);

            if !self.show_ends {
                println!(
                    "{} {} {: <8} {: <8} {}",
                    date_str, start_time, duration_str, total_duration_str, tags_str
                );
            } else {
                println!(
                    "{} {} {} {: <8} {: <8} {}",
                    date_str, start_time, end_time, duration_str, total_duration_str, tags_str
                );
            }
        }
    }
}

fn default_start() -> DateTime<Local> {
    Local::today().and_hms(0, 0, 0)
}

fn default_end() -> DateTime<Local> {
    Local::now()
}
