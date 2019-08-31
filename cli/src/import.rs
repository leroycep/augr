mod line_format;

use augr_core::{Patch, Timesheet};
use clap::arg_enum;
use std::error::Error;
use structopt::StructOpt;

arg_enum! {
    /// List of formats that can be imported
    #[derive(Copy, Clone, Debug)]
    enum Format {
        OriginalLineFormat,
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
    pub fn exec(&self, _timesheet: &Timesheet) -> Result<Vec<Patch>, Box<dyn Error>> {
        let patches = match self.format {
            Format::OriginalLineFormat => line_format::import(&self.path).map_err(Box::new)?,
        };
        Ok(patches)
    }
}
