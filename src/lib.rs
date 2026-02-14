pub mod completion;
pub mod error;
pub mod history;
pub mod id;

mod core;
pub use completion::{Completion, Shell};
pub use core::*;
pub use history::History;

pub(crate) mod arg_context;
pub(crate) mod parsed_flag;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

#[cfg(any(feature = "clap-3", feature = "clap-4"))]
mod generate;
#[cfg(any(feature = "clap-3", feature = "clap-4"))]
pub use generate::Config;
#[cfg(any(feature = "clap-3", feature = "clap-4"))]
pub use generate::generate;
#[cfg(any(feature = "clap-3", feature = "clap-4"))]
pub use generate::generate_default;
