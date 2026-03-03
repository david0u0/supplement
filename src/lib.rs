pub mod completion;
pub mod error;
pub mod history;
pub mod id;

pub mod core;
pub use completion::{Completion, CompletionGroup};
pub use history::History;

pub(crate) mod arg_context;
pub(crate) mod parsed_flag;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub mod helper {
    //! Helper functions/macros to make your life easier.

    pub use supplement_proc_macro::id;
}

#[cfg(any(feature = "clap-3", feature = "clap-4"))]
mod generate;
#[cfg(any(feature = "clap-3", feature = "clap-4"))]
pub use generate::Config;
#[cfg(any(feature = "clap-3", feature = "clap-4"))]
pub use generate::generate;

/// Enum to represent different shell. Use [`str::parse`] to create it.
///
/// ```rust
/// use supplement::Shell;
/// let shell: Shell = "fish".parse().unwrap();
/// assert_eq!(shell, Shell::Fish);
/// ```
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum Shell {
    Zsh,
    Fish,
    Bash,
}
impl std::str::FromStr for Shell {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let ret = match s {
            "zsh" => Shell::Zsh,
            "fish" => Shell::Fish,
            "bash" => Shell::Bash,
            _ => return Err(format!("Unknown shell {}", s)),
        };

        Ok(ret)
    }
}
