use crate::parsed_flag::Error as ParsedFlagError;

#[derive(Debug)]
pub enum Error {
    ParsedFlag(ParsedFlagError),
    BoolFlagEqualsValue(String),
    FlagNotFound(String),
    NoPossibleCompletion,
}

impl From<ParsedFlagError> for Error {
    fn from(value: ParsedFlagError) -> Self {
        Error::ParsedFlag(value)
    }
}
