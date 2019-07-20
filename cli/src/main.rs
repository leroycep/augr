mod config;
mod database;
mod chart;
mod start;
mod summary;
mod sync_folder_db;
mod tags;
mod time_input;
mod timesheet;

use snafu::{ErrorCompat, ResultExt, Snafu};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "augr")]
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

    #[structopt(name = "chart")]
    Chart(chart::Cmd),

    #[structopt(name = "tags")]
    Tags(tags::TagsCmd),
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error getting config: {}", source))]
    GetConfig { source: config::Error },

    #[snafu(display("Error reading data: {}", source))]
    ReadData { source: sync_folder_db::Error },

    #[snafu(display("Error writing data: {}", source))]
    WriteData { source: sync_folder_db::Error },
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("An error occured: {}", e);
            if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                eprintln!("{}", backtrace);
            }
        }
    }
}

fn run() -> Result<(), Error> {
    let opt = Opt::from_args();

    let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "augr").unwrap();
    let conf_file = proj_dirs.config_dir().join("config.toml");

    let conf = config::load_config(&conf_file).context(GetConfig {})?;

    let mut db = sync_folder_db::SyncFolderDB::load(&conf.sync_folder, conf.device_id)
        .context(ReadData {})?;

    match opt.cmd.unwrap_or(Command::default()) {
        Command::Start(subcmd) => subcmd.exec(&mut db),
        Command::Summary(subcmd) => subcmd.exec(&db),
        Command::Chart(subcmd) => subcmd.exec(&db),
        Command::Tags(subcmd) => subcmd.exec(&db),
    }

    db.save().context(WriteData {})?;

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
