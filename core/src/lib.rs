mod action;
mod patch;

pub use action::{Action, ActionRef};
pub use patch::Patch;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
