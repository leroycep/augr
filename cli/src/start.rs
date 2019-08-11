use crate::{database::DataBase, time_input::parse_default_local, timesheet::Tag};
use augr_core::{Patch, Timesheet};
use chrono::{DateTime, Local, Utc};
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
    pub fn exec(&self, _timesheet: &Timesheet) -> Vec<Patch> {
        let event_ref = uuid::Uuid::new_v4().to_string();
        let now = self
            .time
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now());
        let tags = self.tags.iter().cloned().collect();

        vec![Patch::new().create_event(event_ref, now, tags)]
    }
}
