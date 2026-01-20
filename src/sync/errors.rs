use thiserror::Error;

/// Sync-specific errors
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("OpenSRS API error: {0}")]
    OpenSrsError(#[from] crate::opensrs::OpenSrsError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Domain sync failed: {domain} - {reason}")]
    DomainSyncFailed { domain: String, reason: String },
}

pub type Result<T> = std::result::Result<T, SyncError>;
