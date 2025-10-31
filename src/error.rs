/// Error type for VMF parsing operations
#[derive(Debug)]
pub enum VMFError {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    ParseError(String),
}

impl From<std::io::Error> for VMFError {
    fn from(err: std::io::Error) -> Self {
        VMFError::IoError(err)
    }
}

impl From<std::str::Utf8Error> for VMFError {
    fn from(err: std::str::Utf8Error) -> Self {
        VMFError::Utf8Error(err)
    }
}

impl std::fmt::Display for VMFError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMFError::IoError(err) => write!(f, "IO error: {}", err),
            VMFError::Utf8Error(err) => write!(f, "UTF-8 error: {}", err),
            VMFError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for VMFError {}
