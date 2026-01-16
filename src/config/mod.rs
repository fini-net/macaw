//! Configuration management for Macaw
//!
//! This module handles loading configuration from environment variables,
//! including OpenSRS API credentials retrieved via fnox.

use std::env;

/// OpenSRS API credentials
#[derive(Debug, Clone)]
pub struct OpenSrsCredentials {
    pub username: String,
    pub credential: String,
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

impl OpenSrsCredentials {
    /// Load OpenSRS credentials from environment variables
    ///
    /// Expected environment variables:
    /// - `OPENSRS_USERNAME`: OpenSRS API username
    /// - `OPENSRS_CREDENTIAL`: OpenSRS API credential (not password!)
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::MissingEnvVar` if required variables are not set.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use macaw::config::{OpenSrsCredentials, Result};
    ///
    /// fn main() -> Result<()> {
    ///     let creds = OpenSrsCredentials::from_env()?;
    ///     println!("Username: {}", creds.username);
    ///     Ok(())
    /// }
    /// ```
    pub fn from_env() -> Result<Self> {
        let username = env::var("OPENSRS_USERNAME")
            .map_err(|_| ConfigError::MissingEnvVar("OPENSRS_USERNAME".to_string()))?;

        let credential = env::var("OPENSRS_CREDENTIAL")
            .map_err(|_| ConfigError::MissingEnvVar("OPENSRS_CREDENTIAL".to_string()))?;

        // Validate non-empty
        if username.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "OPENSRS_USERNAME cannot be empty".to_string(),
            ));
        }

        if credential.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "OPENSRS_CREDENTIAL cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            username: username.trim().to_string(),
            credential: credential.trim().to_string(),
        })
    }

    /// Check if credentials are available in environment without loading
    ///
    /// Useful for conditional features or early validation.
    pub fn available() -> bool {
        env::var("OPENSRS_USERNAME").is_ok() && env::var("OPENSRS_CREDENTIAL").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_credentials() {
        unsafe {
            env::remove_var("OPENSRS_USERNAME");
            env::remove_var("OPENSRS_CREDENTIAL");
        }

        let result = OpenSrsCredentials::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_available() {
        unsafe {
            env::remove_var("OPENSRS_USERNAME");
            env::remove_var("OPENSRS_CREDENTIAL");
        }
        assert!(!OpenSrsCredentials::available());

        unsafe {
            env::set_var("OPENSRS_USERNAME", "test");
            env::set_var("OPENSRS_CREDENTIAL", "test");
        }
        assert!(OpenSrsCredentials::available());

        unsafe {
            env::remove_var("OPENSRS_USERNAME");
            env::remove_var("OPENSRS_CREDENTIAL");
        }
    }
}
