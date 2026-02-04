pub mod error;
pub mod id;
pub mod info;

mod generate;
pub mod history;
mod utils;
pub use generate::generate;
pub use history::History;
pub use utils::*;

pub(crate) mod arg_context;
pub(crate) mod parsed_flag;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

#[derive(Debug, Eq, PartialEq, Clone)]
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
