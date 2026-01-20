use super::auth::generate_signature;
use super::error::Result;
use super::types::{ClientConfig, GetDomainRequest, GetDomainsByExpireDateRequest};
use super::xml::{
    deserialize_domain_all_info, deserialize_response, serialize_get_domain_request,
    serialize_request,
};

/// OpenSRS API client
pub struct OpenSrsClient {
    config: ClientConfig,
    agent: ureq::Agent,
}

impl OpenSrsClient {
    /// Create a new OpenSRS client with the given configuration
    pub fn new(config: ClientConfig) -> Self {
        let agent = ureq::agent();

        Self { config, agent }
    }

    /// Send a request to the OpenSRS API
    ///
    /// This handles authentication, XML serialization, and response parsing.
    pub(crate) fn send_request(
        &self,
        request: &GetDomainsByExpireDateRequest,
    ) -> Result<super::types::GetDomainsByExpireDateResponse> {
        // Serialize to XML
        let xml = serialize_request(request)?;

        // Generate MD5 signature
        let signature = generate_signature(&xml, &self.config.credential);

        // Build and send HTTP request
        // Note: Content-Length is set automatically by ureq
        let mut response = self
            .agent
            .post(self.config.environment.endpoint())
            .header("Content-Type", "text/xml")
            .header("X-Username", &self.config.username)
            .header("X-Signature", &signature)
            .send(xml.as_str())?;

        // Parse response
        let response_xml = response.body_mut().read_to_string()?;
        let parsed_response = deserialize_response(&response_xml)?;

        Ok(parsed_response)
    }

    /// Send a get_domain request to the OpenSRS API
    ///
    /// This handles authentication, XML serialization, and response parsing for domain queries.
    pub(crate) fn send_get_domain_request(
        &self,
        request: &GetDomainRequest,
    ) -> Result<super::types::GetDomainAllInfoResponse> {
        // Serialize to XML
        let xml = serialize_get_domain_request(request)?;

        // Generate MD5 signature
        let signature = generate_signature(&xml, &self.config.credential);

        // Build and send HTTP request
        let mut response = self
            .agent
            .post(self.config.environment.endpoint())
            .header("Content-Type", "text/xml")
            .header("X-Username", &self.config.username)
            .header("X-Signature", &signature)
            .send(xml.as_str())?;

        // Parse response
        let response_xml = response.body_mut().read_to_string()?;
        let parsed_response = deserialize_domain_all_info(&response_xml)?;

        Ok(parsed_response)
    }
}
