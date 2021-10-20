use crate::{Meta, Patch, PatchRef, Store};
use snafu::{ResultExt, Snafu};
use std::{
    fs::{create_dir_all, read_to_string, OpenOptions},
    io::Write,
    path::PathBuf,
};
use toml;

#[derive(Debug)]
pub struct SyncFolderStore {
    /// Whether the repository should create a new file if one is not found
    init: bool,
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

    #[snafu(display("IO error: {}", source))]
    IOError { source: std::io::Error },
}

impl SyncFolderStore {
    pub fn new(root_folder: PathBuf, device_id: String) -> Self {
        Self {
            init: false,
            device_id,
            patch_folder: root_folder.join("patches"),
            root_folder,
        }
    }

    pub fn should_init(mut self, should_init: bool) -> Self {
        self.init = should_init;
        self
    }

    fn meta_file_path(&self) -> PathBuf {
        self.root_folder
            .join("meta")
            .join(self.device_id.clone())
            .with_extension("toml")
    }

    pub fn get_other_metas(
        &self,
    ) -> Result<impl Iterator<Item = Result<Meta, SyncFolderStoreError>>, SyncFolderStoreError>
    {
        let meta_folder = self.root_folder.join("meta");
        let meta_file = self.meta_file_path();

        if !meta_folder.exists() {
            create_dir_all(&meta_folder).context(IOError {})?;
        }

        let sync_folder_items = meta_folder
            .read_dir()
            .context(ReadFile { path: meta_folder })?;

        let iter = sync_folder_items
            .filter_map(|d| d.ok())
            .filter(move |dir_entry| dir_entry.path() != meta_file)
            .map(|dir_entry| {
                let path = dir_entry.path();
                let contents = read_to_string(&path).context(ReadFile { path: path.clone() })?;

                let meta = toml::de::from_str(&contents).context(DeserializeMeta {
                    device_id: path.display().to_string(),
                })?;

                Ok(meta)
            });
        Ok(iter)
    }
}

impl Store for SyncFolderStore {
    type Error = SyncFolderStoreError;

    #[cfg_attr(feature = "flame_it", flame)]
    fn get_meta(&self) -> Result<Meta, Self::Error> {
        let path = self.meta_file_path();

        if path.exists() || !self.init {
            let contents = read_to_string(&path).context(ReadFile { path })?;

            let meta = toml::de::from_str(&contents).context(DeserializeMeta {
                device_id: self.device_id.clone(),
            })?;

            Ok(meta)
        } else {
            Ok(Meta::new())
        }
    }

    fn save_meta(&mut self, meta: &Meta) -> Result<(), Self::Error> {
        let contents = toml::ser::to_vec(&meta).context(SerializeMeta {
            device_id: self.device_id.clone(),
        })?;

        let path = self.meta_file_path();

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                create_dir_all(parent).context(WriteFile { path: parent })?;
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.clone())
            .context(WriteFile { path: path.clone() })?;

        file.write_all(contents.as_slice())
            .context(WriteFile { path: path.clone() })?;

        Ok(())
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn get_patch(&self, patch_ref: &PatchRef) -> Result<Patch, Self::Error> {
        let path = self
            .patch_folder
            .join(patch_ref.to_string())
            .with_extension("toml");

        let contents = load_file_contents(&path).context(ReadFile { path })?;

        let patch = toml::de::from_str(&contents).context(DeserializePatch {
            patch_ref: patch_ref.to_string(),
        })?;

        Ok(patch)
    }

    fn add_patch(&mut self, patch: &Patch) -> Result<(), Self::Error> {
        let patch_ref = patch.patch_ref().to_string();
        let path = self.patch_folder.join(&patch_ref).with_extension("toml");

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                create_dir_all(parent).context(WriteFile { path: parent })?;
            }
        }

        let contents = toml::ser::to_vec(patch).context(SerializeMeta {
            device_id: self.device_id.clone(),
        })?;

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.clone())
            .context(WriteFile { path: path.clone() })?;

        file.write_all(contents.as_slice())
            .context(WriteFile { path: path.clone() })?;

        Ok(())
    }
}

#[cfg_attr(feature = "flame_it", flame)]
fn load_file_contents(path: &std::path::Path) -> Result<String, std::io::Error> {
    read_to_string(&path)
}
