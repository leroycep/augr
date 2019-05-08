mod show_week;
mod summary;
mod start;

use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "timetrack")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "start")]
    Start(start::StartCmd),

    #[structopt(name = "summary")]
    Summary(summary::SummaryCmd),

    #[structopt(name = "week")]
    Week(show_week::ShowWeekCmd),
}

fn main() {
    let opt = Opt::from_args();

    let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "timetracker").unwrap();
    let data_file = proj_dirs.data_dir().join("timesheet.csv");

    let mut timesheet = load_timesheet(&data_file);

    match opt.cmd.unwrap_or(Command::default()) {
        Command::Start(subcmd) => subcmd.exec(&mut timesheet),
        Command::Summary(subcmd) => subcmd.exec(&timesheet),
        Command::Week(subcmd) => subcmd.exec(&timesheet),
    }

    save_timesheet(&data_file, &timesheet);
}

#[derive(Debug)]
pub struct Timesheet {
    transitions: BTreeMap<DateTime<Utc>, HashSet<Tag>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Tag(String);

fn load_timesheet(path: &Path) -> Timesheet {
    use std::io::Read;

    let mut timesheet = Timesheet::new();
    if !path.exists() {
        return timesheet;
    }

    let mut rdr = std::fs::OpenOptions::new().read(true).open(path).unwrap();
    let mut contents = String::new();
    rdr.read_to_string(&mut contents).unwrap();

    for line in contents.lines() {
        let mut cols = line.split(' ');
        let time = cols.next().unwrap().parse().unwrap();
        let tags = cols.map(|x| Tag(x.into())).collect();
        timesheet.transitions.insert(time, tags);
    }
    timesheet
}

fn save_timesheet(path: &Path, timesheet: &Timesheet) {
    use std::io::Write;

    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut wtr = std::fs::OpenOptions::new().write(true).open(path).unwrap();

    for (start_time, tags) in timesheet.transitions.iter() {
        write!(wtr, "{}", start_time.to_rfc3339()).unwrap();
        for t in tags {
            write!(wtr, " {}", t.0).unwrap();
        }
        wtr.write(b"\n").unwrap();
    }
    wtr.flush().unwrap();
}

fn tags_at_time<'ts>(
    timesheet: &'ts Timesheet,
    datetime: &DateTime<Utc>,
) -> Option<&'ts HashSet<Tag>> {
    timesheet
        .transitions
        .range(..datetime)
        .map(|(_time, tags)| tags)
        .last()
}

fn format_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let mins = duration.num_minutes() - (hours * 60);
    if hours < 1 {
        format!("{}m", mins)
    } else {
        format!("{}h {}m", hours, mins)
    }
}

impl Timesheet {
    pub fn new() -> Timesheet {
        Self {
            transitions: BTreeMap::new(),
        }
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Default for Command {
    fn default() -> Self {
        Command::Summary(summary::SummaryCmd::default())
    }
}
