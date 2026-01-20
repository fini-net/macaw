use super::errors::{Result, SyncError};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration for sync from domains.toml
#[derive(Debug, Deserialize)]
pub struct DomainsConfig {
    pub domain_groups: HashMap<String, Vec<String>>,
}

impl DomainsConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: DomainsConfig = toml::from_str(&contents)?;

        // Validate that we have at least one domain group
        if config.domain_groups.is_empty() {
            return Err(SyncError::ConfigError(
                "No domain groups found in configuration".to_string(),
            ));
        }

        Ok(config)
    }

    /// Get all domain groups
    pub fn groups(&self) -> &HashMap<String, Vec<String>> {
        &self.domain_groups
    }

    /// Get domains for a specific customer
    pub fn get_customer_domains(&self, customer: &str) -> Option<&Vec<String>> {
        self.domain_groups.get(customer)
    }

    /// Get all customers
    pub fn customers(&self) -> Vec<String> {
        self.domain_groups.keys().cloned().collect()
    }
}
