use crate::{database::DataBase, time_input::parse_default_local, timesheet::Tag};
use chrono::{DateTime, Local, Utc};
use std::collections::BTreeSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct StartCmd {
    /// The time when you started
    #[structopt(long = "time", parse(try_from_os_str = "parse_default_local"))]
    time: Option<DateTime<Local>>,

    /// A list of tags showing what you are doing
    tags: Vec<String>,
}

impl StartCmd {
    pub fn exec<DB: DataBase>(&self, db: &mut DB) {
        let now = self
            .time
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now());
        let tags: BTreeSet<Tag> = self.tags.iter().cloned().map(Tag::from).collect();

        db.insert_event(now, tags);
    }
}
