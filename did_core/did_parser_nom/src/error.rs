use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    InvalidInput(&'static str),
    ParserError(Box<dyn std::error::Error + 'static>),
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::InvalidInput(_) => None,
            ParseError::ParserError(err) => Some(err.as_ref()),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidInput(input) => write!(f, "Invalid input: {}", input),
            ParseError::ParserError(error) => write!(f, "Parsing library error: {}", error),
        }
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for ParseError {
    fn from(err: nom::Err<nom::error::Error<&str>>) -> Self {
        ParseError::ParserError(err.to_owned().into())
    }
}
