mod meta;
mod patch;
mod store;

pub use crate::meta::Meta;
pub use crate::patch::{Patch, PatchRef};
pub use crate::store::{Store};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
