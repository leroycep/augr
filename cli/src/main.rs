mod chart;
mod config;
mod start;
mod summary;
mod tags;
mod time_input;

use augr_core::{
    repository::{timesheet::Error as Conflict, Error as RepositoryError, Repository},
    store::{SyncFolderStore, SyncFolderStoreError},
};
use snafu::{ErrorCompat, ResultExt, Snafu};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "augr")]
struct Opt {
    /// Use the config file at the specified path. Defaults to `$XDG_CONFIG_HOME/augr/config.toml`.
    #[structopt(long = "config")]
    config: Option<PathBuf>,

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

    #[snafu(display("Errors reading repository: {:?}", errors))]
    ReadRepository {
        errors: Vec<RepositoryError<SyncFolderStoreError>>,
    },

    #[snafu(display("Conflicts while merging patches: {:?}", conflicts))]
    MergeConflicts { conflicts: Vec<Conflict> },
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

    let conf_file = match opt.config {
        Some(config_path) => config_path,
        None => {
            let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "augr").unwrap();
            proj_dirs.config_dir().join("config.toml")
        }
    };

    let conf = config::load_config(&conf_file).context(GetConfig {})?;

    let store = SyncFolderStore::new(conf.sync_folder.into(), conf.device_id).should_init(true);
    let mut repo = Repository::from_store(store).unwrap();
    let eventgraph = repo.timesheet();
    let timesheet = eventgraph
        .flatten()
        .map_err(|conflicts| Error::MergeConflicts { conflicts })?;

    match opt.cmd.unwrap_or(Command::default()) {
        Command::Start(subcmd) => {
            let patches = subcmd.exec(&timesheet);
            for patch in patches {
                let patch_ref = String::new();
                let res = repo.add_patch(patch).unwrap();
                println!("{}: {:?}", patch_ref, res);
            }
        }
        Command::Summary(subcmd) => subcmd.exec(&timesheet),
        Command::Chart(subcmd) => subcmd.exec(&timesheet),
        Command::Tags(subcmd) => subcmd.exec(&timesheet),
    };

    repo.save_meta().unwrap();

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
