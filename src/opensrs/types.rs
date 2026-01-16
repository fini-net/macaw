use serde::{Deserialize, Serialize};

/// OpenSRS API environment (test or production)
#[derive(Debug, Clone)]
pub enum Environment {
    /// Test environment (OT&E - Operational Test & Evaluation)
    Test,
    /// Production environment
    Production,
}

impl Environment {
    /// Get the API endpoint URL for this environment
    pub fn endpoint(&self) -> &str {
        match self {
            Environment::Production => "https://rr-n1-tor.opensrs.net:55443",
            Environment::Test => "https://horizon.opensrs.net:55443",
        }
    }
}

/// Configuration for the OpenSRS client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// OpenSRS username (reseller account)
    pub username: String,
    /// OpenSRS API credential (private key)
    pub credential: String,
    /// Environment to use (test or production)
    pub environment: Environment,
}

/// XCP protocol envelope structure
#[derive(Debug, Serialize, Deserialize)]
pub struct OpsEnvelope<T> {
    pub header: OpsHeader,
    pub body: OpsBody<T>,
}

/// XCP protocol header
#[derive(Debug, Serialize, Deserialize)]
pub struct OpsHeader {
    pub version: String,
}

/// XCP protocol body
#[derive(Debug, Serialize, Deserialize)]
pub struct OpsBody<T> {
    pub data_block: T,
}

/// Request to get domains by expiration date
#[derive(Debug, Serialize)]
pub struct GetDomainsByExpireDateRequest {
    pub protocol: String,
    pub object: String,
    pub action: String,
    pub attributes: GetDomainsByExpireDateAttrs,
}

/// Attributes for get_domains_by_expiredate request
#[derive(Debug, Serialize)]
pub struct GetDomainsByExpireDateAttrs {
    pub exp_from: String,
    pub exp_to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
}

/// Response from get_domains_by_expiredate
#[derive(Debug, Deserialize)]
pub struct GetDomainsByExpireDateResponse {
    pub is_success: bool,
    pub response_code: String,
    pub response_text: String,
    pub attributes: GetDomainsByExpireDateResponseAttrs,
}

/// Response attributes for get_domains_by_expiredate
#[derive(Debug, Deserialize)]
pub struct GetDomainsByExpireDateResponseAttrs {
    pub page: u32,
    pub total: u32,
    /// 0 = all results returned, 1 = more pages available
    pub remainder: u8,
    #[serde(default)]
    pub exp_domains: Vec<ExpiringDomain>,
}

/// Information about an expiring domain
#[derive(Debug, Deserialize, Clone)]
pub struct ExpiringDomain {
    pub name: String,
    pub expiredate: String,
    pub f_auto_renew: String,
    pub f_let_expire: String,
}
