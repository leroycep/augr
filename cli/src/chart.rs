use crate::{config::Config, summary::get_segments};
use anyhow::Result;
use chrono::{offset::TimeZone, Datelike, Local, NaiveDate};
use std::collections::BTreeSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "chart")]
pub struct Cmd {
    /// A list of filter_tags to filter against
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
        let filter_tags: BTreeSet<String> = self.tags.iter().cloned().collect();

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

        let mut prev_date_opt: Option<chrono::Date<Local>> = None;
        let mut prev_month = 0;

        while cur_date <= end_date {
            // Print out the week number once for each week
            let mut should_print_week = true;
            if let Some(prev_date) = prev_date_opt {
                if cur_date.iso_week() == prev_date.iso_week() {
                    should_print_week = false;
                }
            }
            prev_date_opt = Some(cur_date);

            if should_print_week {
                if prev_month != cur_date.month() {
                    println!("{: ^83}", cur_date.format("%B %Y"));
                    print!("           ");
                    for hour in 0..24 {
                        print!("{: <3}", hour);
                    }
                    println!();
                }
                prev_month = cur_date.month();

                print!("{} ", cur_date.format("W%W %d %a"));
            } else {
                print!("{} ", cur_date.format("    %d %a"));
            }
            for section in 0..(24 * 3) {
                let hour = section / 3;
                let minutes = (section % 3) * 20;
                let cur_datetime = cur_date.and_hms(hour, minutes, 0);

                let matches = match segments.range(..cur_datetime).last() {
                    Some((_start_time, cur_tags)) => {
                        if !cur_tags.is_empty() {
                            let mut contains_all_tags = true;

                            for filter_tag in &filter_tags {
                                if !cur_tags.contains(filter_tag) {
                                    contains_all_tags = false;
                                    break;
                                }
                            }

                            contains_all_tags
                        } else {
                            false
                        }
                    }
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
