mod auth;
mod client;
mod domain;
mod error;
mod types;
mod xml;

// Public exports
pub use client::OpenSrsClient;
pub use error::{OpenSrsError, Result};
pub use types::{ClientConfig, Environment, ExpiringDomain};
