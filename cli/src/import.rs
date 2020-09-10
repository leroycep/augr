mod patches;

use augr_core::{Patch, Timesheet};
use clap::arg_enum;
use std::error::Error;
use structopt::StructOpt;

arg_enum! {
    /// List of formats that can be imported
    #[derive(Copy, Clone, Debug)]
    enum Format {
        Patches,
    }
}

#[derive(StructOpt, Debug)]
pub struct ImportCmd {
    /// The format that is being imported
    #[structopt(possible_values = &Format::variants(), case_insensitive = true)]
    format: Format,

    /// Path to data to import
    path: String,
}

impl ImportCmd {
    pub fn exec(&self, config: &crate::config::Config) -> anyhow::Result<()> {
        match self.format {
            Format::Patches => patches::import(config, &self.path),
        }
    }
}
