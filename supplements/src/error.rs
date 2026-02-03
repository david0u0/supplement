use crate::parsed_flag::Error as ParsedFlagError;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    ParsedFlag(ParsedFlagError),
    BoolFlagEqualsValue(String),
    FlagNotFound(String),
    NoPossibleCompletion,
    NoSubcommandAtAll(String), // try to find subcommand where there isn't any
    NoArgAtAll(String),        // try to provide arg where there isn't any
    ArgsTooShort,
    ArgsTooLong(String),
    SubCommandNotFound(String),
}

impl From<ParsedFlagError> for Error {
    fn from(value: ParsedFlagError) -> Self {
        Error::ParsedFlag(value)
    }
}
