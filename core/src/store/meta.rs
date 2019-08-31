use crate::PatchRef;
use serde::{Deserialize, Serialize};

type Set<T> = std::collections::HashSet<T>;

#[derive(Default, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Meta {
    /// The patches that this Meta file depends on, which may exclude patches
    /// that are referenced as ancestors of some patch that is included.
    patches: Set<PatchRef>,
}

impl Meta {
    pub fn new() -> Self {
        Self {
            patches: Set::new(),
        }
    }

    pub fn add_patch(&mut self, patch_ref: PatchRef) {
        self.patches.insert(patch_ref);
    }

    pub fn patches(&self) -> impl Iterator<Item = &PatchRef> {
        self.patches.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn read_from_toml() {
        let expected = Meta {
            patches: [
                "c10350e8-3f30-4d27-b120-8ee079e256d9",
                "7a826905-7a3e-430d-9d54-5af08ecb482c",
            ]
            .into_iter()
            .map(|s| Uuid::parse_str(s).unwrap())
            .collect(),
        };
        let toml_str = r#"
            patches = ["c10350e8-3f30-4d27-b120-8ee079e256d9", "7a826905-7a3e-430d-9d54-5af08ecb482c"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

}
