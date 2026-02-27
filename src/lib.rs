//! Macaw domain registration backend
//!
//! A domain registration system integrating with OpenSRS API, featuring
//! SQLite caching, multi-customer support, and Authelia authentication.

pub mod config;
pub mod db;
pub mod entities;
pub mod opensrs;
pub mod sync;

// Re-export common types for convenience
pub use config::{ConfigError, OpenSrsCredentials};
pub use opensrs::{
    ClientConfig, ContactInfo as OpenSrsContactInfo, ContactInfoForUpdate, ContactSet, Environment,
    ExpiringDomain, OpenSrsClient, OpenSrsError, SetContactRequest, SetContactResponse,
    SetContactSet,
};
