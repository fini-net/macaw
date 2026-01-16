//! Macaw domain registration backend
//!
//! A domain registration system integrating with OpenSRS API, featuring
//! SQLite caching, multi-customer support, and Authelia authentication.

pub mod config;

// Re-export common types for convenience
pub use config::{ConfigError, OpenSrsCredentials};
