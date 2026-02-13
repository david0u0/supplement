#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// When asking for a flag where there is none.
    /// e.g. `cd --<TAB>`, because there's no flag for `cd` at all
    /// (if we ignore the --help flag).
    UnexpectedFlag,
    /// When a boolean flag got an unexpected `=some_value` part, e.g. `git add -A=value`.
    BoolFlagEqualsValue(String),
    /// When a flag is supposed to have value, but got none. e.g. `git commit -m --amend`.
    /// If you're trying to supply it with a flag-like value, please do `git commit -m=--amend`.
    FlagNoValue(&'static str),
    /// When a flag in CLI command is not found, e.g. `git --xx=<TAB>`.
    /// Or `git --xx lo<TAB>`, though technically the `<TAB>` is not asking for that flag.
    /// `supplements` still needs to know the information about the flag to decide if the
    /// `lo<TAB>` is asking for a subcommand/arg or the flag's value.
    FlagNotFound(String),
    /// Attempt to complete an arg/subcommand, when there is none.
    /// e.g. `cargo build <TAB>`, becuase there's no more arg or subcommand.
    /// Or `cargo build abc -<TAB>`, though technically the `<TAB>` is asking for a flag,
    /// but there is this unexpected arg `abc` in front of it.
    UnexpectedArg(String),
    /// There should be at least 2 args, e.g. `git chec` or `git ''`.
    /// NOTE that the empty string is needed if that position is where user want the completion.
    /// i.e. `git ''` means `git <TAB>`, which will result in all git's subcommands.
    ArgsTooShort,
}

#[cfg(any(feature = "clap-3", feature = "clap-4"))]
#[derive(Debug)]
#[non_exhaustive]
pub enum GenerateError {
    UnprocessedConfigObj(Vec<Vec<String>>),
    IO(std::io::Error),
}
impl From<std::io::Error> for GenerateError {
    fn from(value: std::io::Error) -> Self {
        GenerateError::IO(value)
    }
}
