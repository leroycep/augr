use augr_core::Timesheet;
use std::collections::BTreeSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct TagsCmd {}

impl TagsCmd {
    pub fn exec(&self, timesheet: &Timesheet) {
        let tags = timesheet
            .events()
            .iter()
            .fold(BTreeSet::new(), |acc, x| acc.union(x.1).cloned().collect());

        for tag in tags {
            println!("{}", tag);
        }
    }
}
