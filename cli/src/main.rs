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
    Start { tags: Vec<String> },

    #[structopt(name = "list")]
    List,
}

fn main() {
    let opt = Opt::from_args();

    let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "timetracker").unwrap();
    let data_file = proj_dirs.data_dir().join("timesheet.csv");

    let mut timesheet = load_timesheet(&data_file);

    match opt.cmd.unwrap_or(Command::List) {
        Command::Start { tags } => {
            start_tracking(&mut timesheet, tags.into_iter().map(|s| s.into()).collect())
        }
        Command::List => list_tracking(&timesheet),
    }

    save_timesheet(&data_file, &timesheet);
}

#[derive(Debug)]
struct Timesheet {
    transitions: BTreeMap<DateTime<Utc>, HashSet<Tag>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Tag(String);

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

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

fn start_tracking(timesheet: &mut Timesheet, tags: HashSet<Tag>) {
    let now = Utc::now();
    timesheet.transitions.insert(now, tags);
}

fn list_tracking(timesheet: &Timesheet) {
    let today = Utc::today();
    let mut t_iter = timesheet
        .transitions
        .iter()
        .filter(|x| x.0.date() == today)
        .peekable();

    let mut total_duration = chrono::Duration::seconds(0);

    println!("Start Duration Total     Tags");
    println!(
        "――――― ―――――――― ――――――――  ――――――――"
    );
    while let Some(t) = t_iter.next() {
        let start_time = t.0.with_timezone(&chrono::Local).format("%H:%M");
        let next_time = t_iter.peek().map(|x| x.0.clone()).unwrap_or(Utc::now());
        let tags_str =
            t.1.iter()
                .fold(String::new(), |acc, x| format!("{} {}", acc, x.0));

        let duration = next_time.signed_duration_since(t.0.clone());
        total_duration = total_duration + duration;

        let duration_str = format_duration(duration);
        let total_duration_str = format_duration(total_duration);

        println!(
            "{} {: <8} {: <8} {}",
            start_time, duration_str, total_duration_str, tags_str
        );
    }
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
