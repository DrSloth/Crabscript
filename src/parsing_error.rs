use std::fmt;

pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Debug, PartialEq)]
pub struct ParsingError {
    error_type: ParsingErrorKind,
    line: u64,
}

/// Specifies the type of `Parsing Error`
#[derive(Debug, PartialEq)]
pub enum ParsingErrorKind {
    /// A token, with was required/expected for a certain synatax was not in the token stream
    ExpectedNotFound(String),
    /// A token not fitting in the context occuted
    Unexpected(String),
    /// The file/input ended unexpectedly and the parser don't has any tokens to pase anymore
    UnexpectedEndOfInput,
}

impl ParsingError {
    pub fn new(error_type: ParsingErrorKind, line: u64) -> Self {
        ParsingError { error_type, line }
    }
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ERROR [l. {}]:\t{}",
            self.line,
            match &self.error_type {
                ParsingErrorKind::ExpectedNotFound(s) =>
                    format!("Expected {} but could not find it.", s),
                ParsingErrorKind::Unexpected(s) => format!("An unexpected {} was found.", s),
                ParsingErrorKind::UnexpectedEndOfInput =>
                    format!("The file/command ended unexpectedly. Are you missing something?"),
            }
        )
    }
}
