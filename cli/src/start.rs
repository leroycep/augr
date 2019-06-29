use crate::{
    database::DataBase,
    timesheet::{Tag, Timesheet},
};
use chrono::Utc;
use std::collections::HashSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct StartCmd {
    /// A list of tags showing what you are doing
    tags: Vec<String>,
}

impl StartCmd {
    pub fn exec(&self, timesheet: &mut Timesheet) {
        let now = Utc::now();
        let tags: HashSet<Tag> = self.tags.iter().cloned().map(Tag::from).collect();

        timesheet.insert_transition(now, tags);
    }
}
