#[cfg(feature = "flame_it")]
#[macro_use]
extern crate flamer;

mod config;
mod time_input;

mod start;
mod summary;

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

    /// Shows a table of tracked time; defaults to only showing time tracked today
    #[structopt(no_version, name = "summary")]
    Summary(summary::SummaryCmd),
    // /// Show an ascii art chart of tracked time
    // #[structopt(no_version, name = "chart")]
    // Chart(chart::Cmd),

    // /// Get a list of all the different tags that have been used.
    // #[structopt(no_version, name = "tags")]
    // Tags(tags::TagsCmd),

    // /// Add tags to an existing event
    // #[structopt(no_version, name = "tag")]
    // Tag(tag::Cmd),

    // /// Change when an event started
    // #[structopt(no_version, name = "set-start")]
    // SetStart(set_start::Cmd),

    // /// Import data from version 0.1 of augr
    // #[structopt(no_version, name = "import")]
    // Import(import::ImportCmd),
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    // Load config
    let conf_file = match opt.config {
        Some(config_path) => config_path,
        None => {
            let proj_dirs = directories::ProjectDirs::from("xyz", "geemili", "augr").unwrap();
            proj_dirs.config_dir().join("config.toml")
        }
    };
    let config = config::load_config(&conf_file)?;

    // Run command
    match opt.cmd.unwrap_or_default() {
        Command::Start(subcmd) => subcmd.exec(&config)?,
        Command::Summary(subcmd) => subcmd.exec(&config)?,
    };

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
