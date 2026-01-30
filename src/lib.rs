mod history;
mod utils;
pub use history::*;
pub use utils::*;

pub(crate) mod parsed_flag;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SupplementID(u32, &'static str);
impl SupplementID {
    pub const fn new(id: u32, ident: &'static str) -> Self {
        SupplementID(id, ident)
    }
}

#[derive(Debug)]
pub struct CompResult {
    value: String,
    description: String,
}
impl CompResult {
    pub fn new(value: &str, description: &str) -> Self {
        CompResult {
            value: value.to_owned(),
            description: description.to_owned(),
        }
    }
}
