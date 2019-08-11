pub mod meta;
pub mod patch;
pub mod sync_folder_store;

pub use sync_folder_store::{SyncFolderStore, SyncFolderStoreError};

use self::meta::Meta;
use self::patch::Patch;
use crate::PatchRef;
use std::error::Error;

pub trait Store {
    type Error: Error;

    fn get_meta(&self) -> Result<Meta, Self::Error>;
    fn save_meta(&mut self, meta: &Meta) -> Result<(), Self::Error>;
    fn get_patch(&self, patch_ref: &PatchRef) -> Result<Patch, Self::Error>;
    fn add_patch(&mut self, patch: &Patch) -> Result<(), Self::Error>;
}
