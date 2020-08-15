use crate::{config::Config, summary::get_segments};
use anyhow::Result;
use chrono::{offset::TimeZone, Local, NaiveDate};
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
    pub fn exec(&self, config: &Config) -> Result<()> {
        let tags: BTreeSet<String> = self.tags.iter().cloned().collect();

        let segments = get_segments(config, &Local)?;

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

                let matches = match segments.range(..cur_datetime).last() {
                    Some((_start_time, cur_tags)) => tags.is_subset(cur_tags) && !cur_tags.is_empty(),
                    None => false,
                };

                // Avoid highlighting the entire day
                let in_past = cur_datetime <= now;

                if matches && in_past {
                    print!("█");
                } else {
                    print!(" ");
                }
            }
            println!();
            cur_date = cur_date + chrono::Duration::days(1);
        }

        Ok(())
    }
}
