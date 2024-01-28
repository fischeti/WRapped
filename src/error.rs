use imap;

pub type Result<T> = std::result::Result<T, WrError>;

#[derive(Debug)]
pub enum WrError {
    // IO Error
    IoError(std::io::Error),
    // Error from the IMAP crate
    ImapError(String),
    // Query Error
    QueryError(String),
    // Config Error
    ConfigError(String),
    // Serialization Error
    SerializationError(String),
    // Server Error
    ServerError(String),
    // Mail parsing error
    MailParseError(String),
}

impl std::fmt::Display for WrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WrError::IoError(e) => write!(f, "IO error: {}", e),
            WrError::ImapError(e) => write!(f, "IMAP error: {}", e),
            WrError::QueryError(e) => write!(f, "Query error: {}", e),
            WrError::ConfigError(e) => write!(f, "Config error: {}", e),
            WrError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            WrError::ServerError(e) => write!(f, "Server error: {}", e),
            WrError::MailParseError(e) => write!(f, "Mail parse error: {}", e),
        }
    }
}

impl From<std::io::Error> for WrError {
    fn from(error: std::io::Error) -> Self {
        WrError::IoError(error)
    }
}

impl From<imap::error::Error> for WrError {
    fn from(error: imap::error::Error) -> Self {
        WrError::ImapError(error.to_string())
    }
}

impl From<serde_json::Error> for WrError {
    fn from(error: serde_json::Error) -> Self {
        WrError::SerializationError(error.to_string())
    }
}

impl From<mailparse::MailParseError> for WrError {
    fn from(error: mailparse::MailParseError) -> Self {
        WrError::MailParseError(error.to_string())
    }
}

impl std::error::Error for WrError {}
