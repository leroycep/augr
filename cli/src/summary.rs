use crate::{format_duration, Tag, Timesheet};
use chrono::Utc;
use std::collections::HashSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct SummaryCmd {
    #[structopt(help = "A list of tags to filter with")]
    tags: Vec<String>,
}

impl Default for SummaryCmd {
    fn default() -> Self {
        Self { tags: Vec::new() }
    }
}

impl SummaryCmd {
    pub fn exec(&self, timesheet: &Timesheet) {
        let tags: HashSet<Tag> = self.tags.iter().map(|s| Tag(s.clone())).collect();

        let today = Utc::today();
        let mut t_iter = timesheet
            .transitions
            .iter()
            .filter(|x| x.0.date() == today)
            .filter(|x| x.1.is_superset(&tags))
            .peekable();

        let mut total_duration = chrono::Duration::seconds(0);

        println!("Start Duration Total     Tags");
        println!(
            "――――― ―――――――― ――――――――  ――――――――"
        );
        while let Some(t) = t_iter.next() {
            let start_time = t.0.with_timezone(&chrono::Local).format("%H:%M");
            let next_time = t_iter.peek().map(|x| x.0.clone()).unwrap_or(Utc::now());
            let tags_str =
                t.1.iter()
                    .fold(String::new(), |acc, x| format!("{} {}", acc, x.0));

            let duration = next_time.signed_duration_since(t.0.clone());
            total_duration = total_duration + duration;

            let duration_str = format_duration(duration);
            let total_duration_str = format_duration(total_duration);

            println!(
                "{} {: <8} {: <8} {}",
                start_time, duration_str, total_duration_str, tags_str
            );
        }
    }
}
