mod action;
mod meta;
mod patch;

pub use crate::action::{Action, ActionRef};
pub use crate::meta::Meta;
pub use crate::patch::{Patch, PatchRef};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
