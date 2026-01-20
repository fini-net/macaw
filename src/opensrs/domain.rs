use super::client::OpenSrsClient;
use super::error::{OpenSrsError, Result};
use super::types::{
    DomainAllInfo, ExpiringDomain, GetDomainAttrs, GetDomainRequest, GetDomainsByExpireDateAttrs,
    GetDomainsByExpireDateRequest,
};
use chrono::NaiveDate;

impl OpenSrsClient {
    /// Get domains expiring within a date range
    ///
    /// This method automatically handles pagination and returns all matching domains.
    ///
    /// # Arguments
    ///
    /// * `exp_from` - Start date for expiration range (inclusive)
    /// * `exp_to` - End date for expiration range (inclusive)
    ///
    /// # Returns
    ///
    /// A vector of all domains expiring within the specified date range.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or returns an error response.
    pub fn get_domains_by_expiredate(
        &self,
        exp_from: NaiveDate,
        exp_to: NaiveDate,
    ) -> Result<Vec<ExpiringDomain>> {
        let mut all_domains = Vec::new();
        let mut page = 0u32;

        loop {
            let request = GetDomainsByExpireDateRequest {
                protocol: "XCP".to_string(),
                object: "DOMAIN".to_string(),
                action: "GET_DOMAINS_BY_EXPIREDATE".to_string(),
                attributes: GetDomainsByExpireDateAttrs {
                    exp_from: exp_from.format("%Y-%m-%d").to_string(),
                    exp_to: exp_to.format("%Y-%m-%d").to_string(),
                    limit: Some(40), // Default page size
                    page: Some(page),
                },
            };

            let response = self.send_request(&request)?;

            // Check for API errors
            if !response.is_success {
                return Err(OpenSrsError::ApiError {
                    code: response.response_code,
                    message: response.response_text,
                });
            }

            // Collect domains from this page
            all_domains.extend(response.attributes.exp_domains);

            // Check if more pages exist
            if response.attributes.remainder == 0 {
                break;
            }

            page += 1;
        }

        Ok(all_domains)
    }

    /// Get comprehensive information about a domain
    ///
    /// This method retrieves detailed information including contacts, nameservers,
    /// and TLD-specific data for a domain.
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name to query (e.g., "example.com")
    ///
    /// # Returns
    ///
    /// Complete domain information including registration details, contacts, nameservers, etc.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or returns an error response.
    pub fn get_domain_all_info(&self, domain: &str) -> Result<DomainAllInfo> {
        let request = GetDomainRequest {
            protocol: "XCP".to_string(),
            object: "DOMAIN".to_string(),
            action: "GET".to_string(),
            attributes: GetDomainAttrs {
                domain: domain.to_string(),
                req_type: "all_info".to_string(),
            },
        };

        let response = self.send_get_domain_request(&request)?;

        // Check for API errors
        if !response.is_success {
            return Err(OpenSrsError::ApiError {
                code: response.response_code,
                message: response.response_text,
            });
        }

        Ok(response.attributes)
    }
}
