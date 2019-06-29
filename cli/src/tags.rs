use crate::{database::DataBase, timesheet::Tag};
use std::collections::HashSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct TagsCmd {}

impl TagsCmd {
    pub fn exec<DB: DataBase>(&self, timesheet: &DB) {
        let tags: HashSet<Tag> = timesheet
            .transitions()
            .iter()
            .fold(HashSet::new(), |acc, x| acc.union(x.1).cloned().collect());

        for tag in tags {
            println!("{}", tag.0);
        }
    }
}
