use std::fmt;
use std::mem::discriminant;

pub struct ParsingError {
    error_type: ParsingErrorType,
    line: u64,
}

pub enum ParsingErrorType {
    ExpectedNotFound(String),
    Unexpected(String),
    UnexpecedEnd,
}

impl ParsingError {
    pub fn new(error_type:ParsingErrorType, line:u64) -> Self{
        ParsingError{error_type, line}
    }
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parsing error in line {}:\n\t{}",
            self.line,
            match &self.error_type {
                ParsingErrorType::ExpectedNotFound(s) =>
                    format!("Expected {} but could not find it.", s),
                ParsingErrorType::Unexpected(s) => format!("An unexpected {} was found.", s),
                ParsingErrorType::UnexpecedEnd =>
                    format!("The file/command ended unexpectedly. Are you missing something?"),
            }
        )
    }
}

impl std::cmp::PartialEq for ParsingError{
    fn eq(&self, other: &Self) -> bool {
        discriminant(&self.error_type) == discriminant(&other.error_type)
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
