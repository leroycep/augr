use crate::{database::DataBase, timesheet::Tag};
use std::collections::BTreeSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct TagsCmd {}

impl TagsCmd {
    pub fn exec<DB: DataBase>(&self, timesheet: &DB) {
        let tags: BTreeSet<Tag> = timesheet
            .transitions()
            .iter()
            .fold(BTreeSet::new(), |acc, x| acc.union(x.1).cloned().collect());

        for tag in tags {
            println!("{}", tag.0);
        }
    }
}
