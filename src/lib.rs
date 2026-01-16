//! Macaw domain registration backend
//!
//! A domain registration system integrating with OpenSRS API, featuring
//! SQLite caching, multi-customer support, and Authelia authentication.

pub mod config;
pub mod opensrs;

// Re-export common types for convenience
pub use config::{ConfigError, OpenSrsCredentials};
pub use opensrs::{ClientConfig, Environment, ExpiringDomain, OpenSrsClient, OpenSrsError};
