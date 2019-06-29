mod config;
mod database;
mod show_week;
mod start;
mod summary;
mod sync_folder_db;
mod tags;
mod timesheet;

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

    #[structopt(name = "tags")]
    Tags(tags::TagsCmd),
}

fn main() -> Result<(), Box<std::error::Error>> {
    let opt = Opt::from_args();

    let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "timetracker").unwrap();
    let conf_file = proj_dirs.config_dir().join("config.toml");

    let conf = config::load_config(&conf_file)?;

    let mut db = sync_folder_db::SyncFolderDB::load(&conf.sync_folder, conf.device_id)?;

    match opt.cmd.unwrap_or(Command::default()) {
        Command::Start(subcmd) => subcmd.exec(&mut db),
        Command::Summary(subcmd) => subcmd.exec(&db),
        Command::Week(subcmd) => subcmd.exec(&db),
        Command::Tags(subcmd) => subcmd.exec(&db),
    }

    db.save()?;

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
