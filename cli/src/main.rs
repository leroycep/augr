mod config;
mod database;
mod show_week;
mod start;
mod summary;
mod sync_folder_db;
mod tags;
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

    #[structopt(name = "tags")]
    Tags(tags::TagsCmd),
}

fn main() -> Result<(), Box<std::error::Error>> {
    let opt = Opt::from_args();

    let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "timetracker").unwrap();
    let conf_file = proj_dirs.config_dir().join("config.toml");

    let conf = config::load_config(&conf_file)?;
    let data_file = conf
        .sync_folder
        .join(conf.device_id)
        .with_extension("unknown");

    let mut timesheet = load_timesheet(&data_file)?;

    match opt.cmd.unwrap_or(Command::default()) {
        Command::Start(subcmd) => subcmd.exec(&mut timesheet),
        Command::Summary(subcmd) => subcmd.exec(&timesheet),
        Command::Week(subcmd) => subcmd.exec(&timesheet),
        Command::Tags(subcmd) => subcmd.exec(&timesheet),
    }

    save_timesheet(&data_file, &timesheet)?;

    Ok(())
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
