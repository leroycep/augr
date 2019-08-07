use crate::{Event, Meta, Patch, Repository, Store};
use snafu::{ResultExt, Snafu};
use std::{fs::read_to_string, path::PathBuf};
use toml;

pub struct SyncFolderStore {
    meta_folder: PathBuf,
    patch_folder: PathBuf,
}

#[derive(Debug, Snafu)]
pub enum SyncFolderStoreError {
    #[snafu(display("Unable to deserialize meta {}: {}", device_id, source))]
    DeserializeMeta {
        source: toml::de::Error,
        device_id: String,
    },

    #[snafu(display("Unable to deserialize meta {}: {}", patch_ref, source))]
    DeserializePatch {
        source: toml::de::Error,
        patch_ref: String,
    },

    #[snafu(display("Unable to read file {}: {}", path.display(), source))]
    ReadFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Unable to write file {}: {}", path.display(), source))]
    WriteFile {
        source: std::io::Error,
        path: PathBuf,
    },
}

impl SyncFolderStore {
    pub fn new(root_folder: PathBuf) -> Self {
        Self {
            meta_folder: root_folder.join("meta"),
            patch_folder: root_folder.join("patches"),
        }
    }
}

impl Store for SyncFolderStore {
    type Error = SyncFolderStoreError;

    fn get_device_meta(&self, device_id: &str) -> Result<Meta, Self::Error> {
        let path = self.meta_folder.join(device_id).with_extension("toml");

        let contents = read_to_string(&path).context(ReadFile { path })?;

        let meta = toml::de::from_str(&contents).context(DeserializeMeta {
            device_id: device_id.to_string(),
        })?;

        Ok(meta)
    }

    fn get_patch(&self, patch_ref: &str) -> Result<Patch, Self::Error> {
        let path = self.patch_folder.join(patch_ref).with_extension("toml");

        let contents = read_to_string(&path).context(ReadFile { path })?;

        let patch = toml::de::from_str(&contents).context(DeserializePatch {
            patch_ref: patch_ref.to_string(),
        })?;

        Ok(patch)
    }
}
