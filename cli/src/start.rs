use crate::{database::DataBase, timesheet::Tag};
use chrono::Utc;
use std::collections::HashSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct StartCmd {
    /// A list of tags showing what you are doing
    tags: Vec<String>,
}

impl StartCmd {
    pub fn exec<DB: DataBase>(&self, db: &mut DB) {
        let now = Utc::now();
        let tags: HashSet<Tag> = self.tags.iter().cloned().map(Tag::from).collect();

        db.insert_transition(now, tags);
    }
}
