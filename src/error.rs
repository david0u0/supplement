use crate::parsed_flag::Error as ParsedFlagError;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    ParsedFlag(ParsedFlagError),
    BoolFlagEqualsValue(String),
    FlagNoValue(&'static str),
    FlagNotFound(String),
    NoPossibleCompletion,
    UnexpectedArg(String), // try to provide arg where there isn't any
    ArgsTooShort,
}

impl From<ParsedFlagError> for Error {
    fn from(value: ParsedFlagError) -> Self {
        Error::ParsedFlag(value)
    }
}
