use augr_core::{Meta, Patch, Store};
use chrono::{DateTime, Utc};
use snafu::{ResultExt, Snafu};
use std::{fs::read_to_string, path::PathBuf};
use toml;

struct SimpleStore {
    meta_folder: PathBuf,
    patch_folder: PathBuf,
    _device_id: String,
}

#[derive(Debug, Snafu)]
pub enum SimpleStoreError {
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

impl SimpleStore {
    pub fn new(root_folder: PathBuf, device_id: String) -> Self {
        Self {
            meta_folder: root_folder.join("meta"),
            patch_folder: root_folder.join("patches"),
            _device_id: device_id,
        }
    }
}

impl Store for SimpleStore {
    type Error = SimpleStoreError;

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

macro_rules! dt {
    ( $dt:expr ) => {{
        $dt.parse::<DateTime<Utc>>().expect("Valid datetime")
    }};
}

macro_rules! svec {
    ( $( $s:expr ),* ) => {
        vec![ $( $s.to_string(), )* ]
    };
}

macro_rules! s {
    ($s:expr) => {
        $s.to_string()
    };
}

macro_rules! meta {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_meta = Meta::new();
            $(
                temp_meta.add_patch($x.to_string());
            )*
            temp_meta
        }
    };
}

#[test]
fn load_patches_into_store() {
    let expected_metas = vec![("laptop", meta!["laptop-1", "laptop-2"])];
    let expected_patches = vec![
        (
            "laptop-patch-1",
            Patch::new()
                .create_event(s!("a"), dt!("2019-07-23T12:00:00Z"), svec!["lunch", "food"])
                .create_event(s!("b"), dt!("2019-07-23T13:00:00Z"), svec!["work"]),
        ),
        (
            "laptop-patch-2",
            Patch::new()
                .remove_start(s!("laptop-patch-1"), s!("a"), dt!("2019-07-23T12:00:00Z"))
                .add_start(s!("laptop-patch-1"), s!("a"), dt!("2019-07-23T12:30:00Z"))
                .remove_tag(s!("laptop-patch-1"), s!("a"), s!("food"))
                .add_tag(s!("laptop-patch-1"), s!("b"), s!("awesome-project")),
        ),
    ];

    let store = SimpleStore::new("tests/basic_repo".into(), s!("laptop"));

    for (device_id, meta) in expected_metas {
        assert_eq!(store.get_device_meta(device_id).unwrap(), meta);
    }
    for (patch_ref, patch) in expected_patches {
        assert_eq!(store.get_patch(patch_ref).unwrap(), patch);
    }
}
