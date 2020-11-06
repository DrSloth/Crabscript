use std::fmt;

//NOTE it might be possible to implement this with &'a str instead of string, but that would be hard

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
    Unexpected {
        unexpected: String,
        expected: Option<String>,
    },
    /// The file/input ended unexpectedly and the parser don't has any tokens to pase anymore
    UnexpectedEndOfInput,
    /// An undefined was tried to be accessed
    UndefinedVariable(String),
}

impl<'a> ParsingError {
    pub fn new(error_type: ParsingErrorKind, line: u64) -> Self {
        ParsingError { error_type, line }
    }

    pub fn unexpected_end_of_input(line: u64) -> Self {
        Self::new(ParsingErrorKind::UnexpectedEndOfInput, line)
    }

    pub fn unexpected(line: u64, unexpected: String) -> Self {
        Self::new(
            ParsingErrorKind::Unexpected {
                unexpected,
                expected: None,
            },
            line,
        )
    }

    pub fn unexpected_expected(line: u64, unexpected: String, expected: String) -> Self {
        Self::new(
            ParsingErrorKind::Unexpected {
                unexpected,
                expected: Some(expected),
            },
            line,
        )
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
                ParsingErrorKind::Unexpected {
                    unexpected,
                    expected,
                } =>
                    if let Some(ex) = expected {
                        format!("Expected an {} found {}.", ex, unexpected)
                    } else {
                        format!("An unexpected {} was found.", unexpected)
                    },
                ParsingErrorKind::UnexpectedEndOfInput =>
                //TODO This should not allocate a string
                    "The file/command ended unexpectedly. Are you missing something?".to_string(),
                ParsingErrorKind::UndefinedVariable(id) =>
                    format!("The variable {} was not defined", id),
            }
        )
    }
}
