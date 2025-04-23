use chumsky::error::{Error as ChumError, Rich};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VMFParserError {
    #[error("VMF Parser Error: {0}")]
    Parser(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<Rich<'_, char>> for VMFParserError {
    fn from(err: Rich<char>) -> Self {
        VMFParserError::Parser(err.to_string())
    }
}

impl From<Rich<'_, &str>> for VMFParserError {
    fn from(err: Rich<&str>) -> Self {
        VMFParserError::Parser(err.to_string())
    }
}
