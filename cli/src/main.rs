mod show_week;
mod start;
mod summary;
mod timesheet;

use structopt::StructOpt;
use timesheet::{load_timesheet, save_timesheet};

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

fn format_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let mins = duration.num_minutes() - (hours * 60);
    if hours < 1 {
        format!("{}m", mins)
    } else {
        format!("{}h {}m", hours, mins)
    }
}

impl Default for Command {
    fn default() -> Self {
        Command::Summary(summary::SummaryCmd::default())
    }
}
