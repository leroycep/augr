use crate::{Meta, Patch, PatchRef, Store};
use snafu::{ResultExt, Snafu};
use std::{fs::read_to_string, path::PathBuf};
use toml;

pub struct SyncFolderStore {
    root_folder: PathBuf,
    patch_folder: PathBuf,
    device_id: String,
}

#[derive(Debug, Snafu)]
pub enum SyncFolderStoreError {
    #[snafu(display("Unable to deserialize meta {}: {}", device_id, source))]
    DeserializeMeta {
        source: toml::de::Error,
        device_id: String,
    },

    #[snafu(display("Unable to serialize meta {}: {}", device_id, source))]
    SerializeMeta {
        source: toml::ser::Error,
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
    pub fn new(root_folder: PathBuf, device_id: String) -> Self {
        Self {
            device_id,
            patch_folder: root_folder.join("patches"),
            root_folder: root_folder,
        }
    }

    fn meta_file_path(&self) -> PathBuf {
        self.root_folder
            .join("meta")
            .join(self.device_id.clone())
            .with_extension("toml")
    }
}

impl Store for SyncFolderStore {
    type Error = SyncFolderStoreError;

    fn get_meta(&self) -> Result<Meta, Self::Error> {
        let path = self.meta_file_path();

        let contents = read_to_string(&path).context(ReadFile { path })?;

        let meta = toml::de::from_str(&contents).context(DeserializeMeta {
            device_id: self.device_id.clone(),
        })?;

        Ok(meta)
    }

    fn save_meta(&mut self, meta: &Meta) -> Result<(), Self::Error> {
        let path = self.meta_file_path();

        let contents = toml::ser::to_string(&meta).context(SerializeMeta {
            device_id: self.device_id.clone(),
        })?;

        std::fs::write(&path, contents).context(WriteFile { path })?;

        Ok(())
    }

    fn get_patch(&self, patch_ref: &str) -> Result<Patch, Self::Error> {
        let path = self.patch_folder.join(patch_ref).with_extension("toml");

        let contents = read_to_string(&path).context(ReadFile { path })?;

        let patch = toml::de::from_str(&contents).context(DeserializePatch {
            patch_ref: patch_ref.to_string(),
        })?;

        Ok(patch)
    }

    fn add_patch(&mut self, patch: &Patch) -> Result<PatchRef, Self::Error> {
        let patch_ref = "hello".to_string();
        let path = self.patch_folder.join(&patch_ref).with_extension("toml");

        let contents = toml::ser::to_string(patch).context(SerializeMeta {
            device_id: self.device_id.clone(),
        })?;

        std::fs::write(&path, contents).context(WriteFile { path })?;

        Ok(patch_ref)
    }
}
