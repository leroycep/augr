mod chart;
mod config;
mod import;
mod set_start;
mod start;
mod summary;
mod tag;
mod untag;
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
#[structopt(name = "augr", about, author)]
struct Opt {
    /// Use the config file at the specified path. Defaults to `$XDG_CONFIG_HOME/augr/config.toml`.
    #[structopt(long = "config")]
    config: Option<PathBuf>,

    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Add an event to the timesheet; start defaults to the current time
    #[structopt(no_version, name = "start")]
    Start(start::StartCmd),

    /// Show a table tracked time; defaults to only showing time tracked today
    #[structopt(no_version, name = "summary")]
    Summary(summary::SummaryCmd),

    /// Show an ascii art chart of tracked time
    #[structopt(no_version, name = "chart")]
    Chart(chart::Cmd),

    /// Get a list of all the different tags that have been used.
    #[structopt(no_version, name = "tags")]
    Tags(tags::TagsCmd),

    /// Add tags to an existing event
    #[structopt(no_version, name = "tag")]
    Tag(tag::Cmd),

    /// Remove tags from an existing event
    #[structopt(no_version, name = "untag")]
    Untag(untag::Cmd),

    /// Change when an event started
    #[structopt(no_version, name = "set-start")]
    SetStart(set_start::Cmd),

    /// Import data from version 0.1 of augr
    #[structopt(no_version, name = "import")]
    Import(import::ImportCmd),
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

    #[snafu(display("Error importing data: {}", source))]
    ImportError { source: Box<dyn std::error::Error> },

    #[snafu(display("Errors synchronizing data: {:?}", errors))]
    SyncError {
        errors: Vec<RepositoryError<SyncFolderStoreError>>,
    },

    #[snafu(display("Error: {}", source))]
    GeneralError { source: Box<dyn std::error::Error> },
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

    // Load config
    let conf_file = match opt.config {
        Some(config_path) => config_path,
        None => {
            let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "augr").unwrap();
            proj_dirs.config_dir().join("config.toml")
        }
    };
    let conf = config::load_config(&conf_file).context(GetConfig {})?;

    // Load store for own data
    let store = SyncFolderStore::new(conf.sync_folder, conf.device_id).should_init(true);
    let mut repo = Repository::from_store(store).unwrap();

    // Synchronize data
    repo.try_sync_data()
        .map_err(|errors| Error::SyncError { errors })?;
    repo.save_meta().unwrap();

    // Convert abstract patch data structure into a more conventional format
    let eventgraph = repo.timesheet();
    let timesheet = eventgraph
        .flatten()
        .map_err(|conflicts| Error::MergeConflicts { conflicts })?;

    // Run command
    match opt.cmd.unwrap_or_default() {
        Command::Start(subcmd) => {
            let patches = subcmd.exec(&timesheet);
            for patch in patches {
                println!("{}", patch.patch_ref());
                repo.add_patch(patch).unwrap();
            }
        }
        Command::Import(subcmd) => {
            let patches = subcmd.exec(&timesheet).context(ImportError {})?;
            for patch in patches {
                println!("{}", patch.patch_ref());
                repo.add_patch(patch).unwrap();
            }
        }
        Command::Summary(subcmd) => subcmd.exec(&timesheet),
        Command::Chart(subcmd) => subcmd.exec(&timesheet),
        Command::Tags(subcmd) => subcmd.exec(&timesheet),
        Command::Tag(subcmd) => {
            let patches = subcmd
                .exec(&timesheet)
                .map_err(|e| Box::new(e).into())
                .context(GeneralError {})?;
            for patch in patches {
                println!("{}", patch.patch_ref());
                repo.add_patch(patch).unwrap();
            }
        }
        Command::Untag(subcmd) => {
            let patches = subcmd
                .exec(&timesheet)
                .map_err(|e| Box::new(e).into())
                .context(GeneralError {})?;
            for patch in patches {
                println!("{}", patch.patch_ref());
                repo.add_patch(patch).unwrap();
            }
        }
        Command::SetStart(subcmd) => {
            let patches = subcmd
                .exec(&timesheet)
                .map_err(|e| Box::new(e).into())
                .context(GeneralError {})?;
            for patch in patches {
                println!("{}", patch.patch_ref());
                repo.add_patch(patch).unwrap();
            }
        }
    };

    // Save which patches this device uses to disk
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
