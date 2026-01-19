use thiserror::Error;

/// Errors that can occur when interacting with the OpenSRS API
#[derive(Debug, Error)]
pub enum OpenSrsError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] ureq::Error),

    /// XML serialization failed
    #[error("XML serialization failed: {0}")]
    XmlSerialize(#[from] quick_xml::Error),

    /// XML deserialization failed
    #[error("XML deserialization failed: {0}")]
    XmlDeserialize(String),

    /// API returned an error response
    #[error("API returned error: {code} - {message}")]
    ApiError { code: String, message: String },

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Invalid date format
    #[error("Invalid date format: {0}")]
    DateFormatError(String),
}

/// Result type alias for OpenSRS operations
pub type Result<T> = std::result::Result<T, OpenSrsError>;
