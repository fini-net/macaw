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
    #[allow(dead_code)]
    pub page: u32,
    #[allow(dead_code)]
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

/// Request to get domain information
#[derive(Debug, Serialize)]
pub struct GetDomainRequest {
    pub protocol: String,
    pub object: String,
    pub action: String,
    pub attributes: GetDomainAttrs,
}

/// Attributes for get_domain request
#[derive(Debug, Serialize)]
pub struct GetDomainAttrs {
    pub domain: String,
    #[serde(rename = "type")]
    pub req_type: String,
}

/// Response from get_domain with all_info type
#[derive(Debug, Deserialize)]
pub struct GetDomainAllInfoResponse {
    pub is_success: bool,
    pub response_code: String,
    pub response_text: String,
    pub attributes: DomainAllInfo,
}

/// Comprehensive domain information
#[derive(Debug, Deserialize, Clone)]
pub struct DomainAllInfo {
    pub domain_name: String,
    #[serde(default)]
    pub domain_expdate: String,
    #[serde(default)]
    pub registry_createdate: String,
    #[serde(default)]
    pub auto_renew: String,
    #[serde(default)]
    pub lock_state: String,
    #[serde(default)]
    pub whois_privacy_state: String,
    #[serde(default)]
    pub registry_domainid: String,
    #[serde(default)]
    pub contact_set: ContactSet,
    #[serde(default)]
    pub nameserver_list: Vec<NameserverInfo>,
    #[serde(default)]
    pub tld_data: TldDataMap,
}

/// Contact set with all four contact types
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ContactSet {
    #[serde(default)]
    pub owner: Option<ContactInfo>,
    #[serde(default)]
    pub admin: Option<ContactInfo>,
    #[serde(default)]
    pub billing: Option<ContactInfo>,
    #[serde(default)]
    pub tech: Option<ContactInfo>,
}

/// Contact information from OpenSRS
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ContactInfo {
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub fax: Option<String>,
    #[serde(default)]
    pub address1: String,
    #[serde(default)]
    pub address2: Option<String>,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub postal_code: String,
    #[serde(default)]
    pub country: String,
}

/// Nameserver information from OpenSRS
#[derive(Debug, Deserialize, Clone, Default)]
pub struct NameserverInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub sortorder: i32,
    #[serde(default)]
    pub ipaddress: Option<String>,
}

/// TLD-specific data map (for .ca, .us, etc.)
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TldDataMap {
    #[serde(default, flatten)]
    pub data: std::collections::HashMap<String, String>,
}

/// Field name mapping between OpenSRS and local DB schema
/// OpenSRS uses different field names than our DB columns
#[allow(dead_code)]
pub mod contact_mapping {
    pub const ORG_NAME: &str = "org_name";
    pub const STATE: &str = "state";
    pub const COUNTRY: &str = "country";

    pub const DB_ORGANIZATION: &str = "organization";
    pub const DB_STATE_PROVINCE: &str = "state_province";
    pub const DB_COUNTRY_CODE: &str = "country_code";
}

/// Request to update domain contacts
#[derive(Debug, Serialize)]
pub struct SetContactRequest {
    pub protocol: String,
    pub object: String,
    pub action: String,
    pub attributes: SetContactAttrs,
}

/// Attributes for set_contact request
#[derive(Debug, Serialize)]
pub struct SetContactAttrs {
    pub domain: String,
    pub contact_set: SetContactSet,
}

/// Contact set for update requests - all four contact types
#[derive(Debug, Serialize, Clone, Default)]
pub struct SetContactSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<ContactInfoForUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<ContactInfoForUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing: Option<ContactInfoForUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech: Option<ContactInfoForUpdate>,
}

/// Contact information for update requests
/// Uses OpenSRS field naming (org_name, state, country)
#[derive(Debug, Serialize, Clone, Default)]
pub struct ContactInfoForUpdate {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_name: Option<String>,
    pub email: String,
    pub phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fax: Option<String>,
    pub address1: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

/// Convert from internal ContactInfo to ContactInfoForUpdate for OpenSRS
impl From<&ContactInfo> for ContactInfoForUpdate {
    fn from(contact: &ContactInfo) -> Self {
        Self {
            first_name: contact.first_name.clone(),
            last_name: contact.last_name.clone(),
            org_name: contact.org_name.clone(),
            email: contact.email.clone(),
            phone: contact.phone.clone(),
            fax: contact.fax.clone(),
            address1: contact.address1.clone(),
            address2: contact.address2.clone(),
            city: contact.city.clone(),
            state: contact.state.clone(),
            postal_code: contact.postal_code.clone(),
            country: contact.country.clone(),
        }
    }
}

/// Convert from OpenSRS ContactInfo to internal ContactInfo
/// Handles field name mapping from OpenSRS to DB schema
impl From<&ContactInfo> for crate::db::contacts::ContactInfo {
    fn from(contact: &ContactInfo) -> Self {
        Self {
            contact_type: String::new(),
            first_name: contact.first_name.clone(),
            last_name: contact.last_name.clone(),
            organization: contact.org_name.clone(),
            email: contact.email.clone(),
            phone: contact.phone.clone(),
            fax: contact.fax.clone(),
            address1: contact.address1.clone(),
            address2: contact.address2.clone(),
            city: contact.city.clone(),
            state_province: contact.state.clone(),
            postal_code: contact.postal_code.clone(),
            country_code: contact.country.clone(),
        }
    }
}

/// Response from set_contact
#[derive(Debug, Deserialize)]
pub struct SetContactResponse {
    pub is_success: bool,
    pub response_code: String,
    pub response_text: String,
}
