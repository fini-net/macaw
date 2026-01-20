use crate::entities::{domains, prelude::Domains};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

/// Domain data for create or update operations
pub struct DomainData {
    pub customer_id: i32,
    pub domain_name: String,
    pub tld: String,
    pub status: String,
    pub registration_date: String,
    pub expiration_date: String,
    pub auto_renew: bool,
    pub transfer_lock: bool,
    pub auth_code: Option<String>,
    pub whois_privacy: bool,
    pub registry_domain_id: Option<String>,
}

/// Create or update a domain
pub async fn create_or_update_domain(
    db: &DatabaseConnection,
    data: DomainData,
) -> Result<domains::Model, DbErr> {
    // Check if domain already exists
    if let Some(existing) = Domains::find()
        .filter(domains::Column::DomainName.eq(&data.domain_name))
        .one(db)
        .await?
    {
        // Update existing domain
        let mut domain: domains::ActiveModel = existing.into();
        domain.customer_id = Set(data.customer_id);
        domain.tld = Set(data.tld);
        domain.status = Set(data.status);
        domain.registration_date = Set(data.registration_date);
        domain.expiration_date = Set(data.expiration_date);
        domain.auto_renew = Set(data.auto_renew);
        domain.transfer_lock = Set(data.transfer_lock);
        domain.auth_code = Set(data.auth_code);
        domain.whois_privacy = Set(data.whois_privacy);
        domain.registry_domain_id = Set(data.registry_domain_id);
        domain.updated_at = Set(chrono::Utc::now().to_rfc3339());

        return domain.update(db).await;
    }

    // Create new domain
    let new_domain = domains::ActiveModel {
        customer_id: Set(data.customer_id),
        domain_name: Set(data.domain_name),
        tld: Set(data.tld),
        status: Set(data.status),
        registration_date: Set(data.registration_date),
        expiration_date: Set(data.expiration_date),
        auto_renew: Set(data.auto_renew),
        transfer_lock: Set(data.transfer_lock),
        auth_code: Set(data.auth_code),
        whois_privacy: Set(data.whois_privacy),
        registry_domain_id: Set(data.registry_domain_id),
        created_at: Set(chrono::Utc::now().to_rfc3339()),
        updated_at: Set(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };

    new_domain.insert(db).await
}

/// Get domain by name
pub async fn get_domain_by_name(
    db: &DatabaseConnection,
    domain_name: &str,
) -> Result<Option<domains::Model>, DbErr> {
    Domains::find()
        .filter(domains::Column::DomainName.eq(domain_name))
        .one(db)
        .await
}

/// Extract TLD from domain name
pub fn extract_tld(domain_name: &str) -> String {
    domain_name.split('.').next_back().unwrap_or("").to_string()
}

/// Map OpenSRS status to database status
pub fn map_opensrs_status(opensrs_status: &str) -> &str {
    match opensrs_status.to_lowercase().as_str() {
        "active" => "active",
        "pending" => "pending",
        "expired" => "grace",
        "redemption" => "redemption",
        _ => "active",
    }
}
