use crate::parsed_flag::Error as ParsedFlagError;

#[derive(Debug)]
pub enum Error {
    ParsedFlag(ParsedFlagError),
    ValueForBoolFlag(String),
    FlagNotFound(String),
}

impl From<ParsedFlagError> for Error {
    fn from(value: ParsedFlagError) -> Self {
        Error::ParsedFlag(value)
    }
}
