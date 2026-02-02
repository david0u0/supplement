use crate::parsed_flag::Error as ParsedFlagError;

#[derive(Debug)]
pub enum Error {
    ParsedFlag(ParsedFlagError),
    BoolFlagEqualsValue(String),
    FlagNotFound(String),
    NoPossibleCompletion,
    ArgsTooShort,
    ArgsTooLong(String),
    SubCommandNotFound(String),
}

impl From<ParsedFlagError> for Error {
    fn from(value: ParsedFlagError) -> Self {
        Error::ParsedFlag(value)
    }
}
