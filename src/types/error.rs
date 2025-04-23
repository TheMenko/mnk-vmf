use thiserror::Error;

#[derive(Error, Debug)]
pub enum VMFError {
    #[error("Failed to parse VMF: {0}")]
    ParseError(String),
}
