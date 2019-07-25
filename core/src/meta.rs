use crate::PatchRef;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct Meta {
    /// The patches that this Meta file depends on, which may exclude patches
    /// that are referenced as ancestors of some patch that is included.
    patches: Vec<PatchRef>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_from_toml() {
        let expected = Meta {
            patches: vec!["laptop-1".to_string(), "laptop-2".to_string()],
        };
        let toml_str = r#"
            patches = ["laptop-1", "laptop-2"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

}
