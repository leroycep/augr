use crate::PatchRef;
use serde::{Deserialize, Serialize};

type Set<T> = std::collections::HashSet<T>;

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
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

    #[test]
    fn read_from_toml() {
        let expected = Meta {
            patches: ["laptop-1", "laptop-2"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        };
        let toml_str = r#"
            patches = ["laptop-1", "laptop-2"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

}
