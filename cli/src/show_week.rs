use crate::{database::DataBase, timesheet::Tag};
use chrono::{offset::TimeZone, Local, NaiveDate, Utc};
use std::collections::BTreeSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "week")]
pub struct ShowWeekCmd {
    /// A list of tags to filter against
    tags: Vec<String>,

    #[structopt(long = "start")]
    start: Option<NaiveDate>,

    #[structopt(long = "end")]
    end: Option<NaiveDate>,
}

impl ShowWeekCmd {
    pub fn exec<DB: DataBase>(&self, timesheet: &DB) {
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
                    .map(|x| tags.is_subset(x) && !x.is_empty())
                    .unwrap_or(false);

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
    }
}
