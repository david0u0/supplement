pub mod error;

mod history;
mod utils;
pub use history::*;
pub use utils::*;

pub(crate) mod parsed_flag;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SupplementID(u32, &'static str);
impl SupplementID {
    pub const fn new(id: u32, ident: &'static str) -> Self {
        SupplementID(id, ident)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Completion {
    pub value: String,
    pub description: String,
}
impl Completion {
    pub fn new(value: &str, description: &str) -> Self {
        Completion {
            value: value.to_owned(),
            description: description.to_owned(),
        }
    }
}
