use crate::{tags_at_time, Tag, Timesheet};
use chrono::Utc;
use structopt::StructOpt;
use std::collections::HashSet;

#[derive(StructOpt, Debug)]
#[structopt(name = "week")]
pub struct ShowWeekCmd {
    #[structopt(help = "A list of tags to filter with")]
    tags: Vec<String>,
}

impl ShowWeekCmd {
    pub fn exec(&self, timesheet: &Timesheet) {
        let tags: HashSet<Tag> = self.tags.iter().map(|s| Tag(s.clone())).collect();

        let today = chrono::Local::today();
        let now = chrono::Local::now();
        let start_date = today - chrono::Duration::days(6);

        let mut cur_date = start_date;

        print!("Day ");
        for hour in 0..24 {
            print!("{: <3}", hour);
        }
        println!();

        while cur_date <= today {
            print!("{} ", cur_date.format("%a"));
            for section in 0..(24 * 3) {
                let hour = section / 3;
                let minutes = (section % 3) * 20;
                let cur_datetime = cur_date.and_hms(hour, minutes, 0);
                let cur_tags = tags_at_time(timesheet, &cur_datetime.with_timezone(&Utc));
                let matches = cur_tags
                    .map(|x| tags.is_subset(x) && !x.is_empty())
                    .unwrap_or(false);

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
