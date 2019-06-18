use crate::timesheet::{Tag, Timesheet};
use std::collections::HashSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct TagsCmd {}

impl TagsCmd {
    pub fn exec(&self, timesheet: &Timesheet) {
        let tags: HashSet<Tag> = timesheet
            .transitions()
            .fold(HashSet::new(), |acc, x| acc.union(x.1).cloned().collect());

        for tag in tags {
            println!("{}", tag.0);
        }
    }
}
