use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "timetrack")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "start")]
    Start { tags: Vec<String> },
}

fn main() {
    let opt = Opt::from_args();

    let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "timetracker").unwrap();
    let data_file = proj_dirs.data_dir().join("timesheet.csv");

    let mut timesheet = load_timesheet(&data_file);

    match opt.cmd {
        Command::Start { tags } => {
            start_tracking(&mut timesheet, tags.into_iter().map(|s| s.into()).collect())
        }
    }

    save_timesheet(&data_file, &timesheet);
}

#[derive(Debug)]
struct Timesheet {
    transitions: BTreeMap<DateTime<Utc>, HashSet<Tag>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Transition(DateTime<Utc>, HashSet<Tag>);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
struct Tag(String);

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

fn load_timesheet(path: &Path) -> Timesheet {
    let mut timesheet = Timesheet::new();
    if !path.exists() {
        return timesheet;
    }
    let mut rdr = csv::Reader::from_path(path).unwrap();
    for result in rdr.deserialize() {
        let transition: Transition = result.unwrap();
        timesheet.transitions.insert(transition.0, transition.1);
    }
    timesheet
}

fn save_timesheet(path: &Path, timesheet: &Timesheet) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut wtr = csv::Writer::from_path(path).unwrap();

    for (start_time, tags) in timesheet.transitions.iter() {
        wtr.serialize(Transition(*start_time, tags.clone()))
            .unwrap();
    }
}

fn start_tracking(timesheet: &mut Timesheet, tags: HashSet<Tag>) {
    let now = Utc::now();
    timesheet.transitions.insert(now, tags);
}

impl Timesheet {
    pub fn new() -> Timesheet {
        Self {
            transitions: BTreeMap::new(),
        }
    }
}
