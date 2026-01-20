use crate::entities::{domains, prelude::Domains};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

/// Create or update a domain
pub async fn create_or_update_domain(
    db: &DatabaseConnection,
    customer_id: i32,
    domain_name: &str,
    tld: &str,
    status: &str,
    registration_date: &str,
    expiration_date: &str,
    auto_renew: bool,
    transfer_lock: bool,
    auth_code: Option<String>,
    whois_privacy: bool,
    registry_domain_id: Option<String>,
) -> Result<domains::Model, DbErr> {
    // Check if domain already exists
    if let Some(existing) = Domains::find()
        .filter(domains::Column::DomainName.eq(domain_name))
        .one(db)
        .await?
    {
        // Update existing domain
        let mut domain: domains::ActiveModel = existing.into();
        domain.customer_id = Set(customer_id);
        domain.tld = Set(tld.to_string());
        domain.status = Set(status.to_string());
        domain.registration_date = Set(registration_date.to_string());
        domain.expiration_date = Set(expiration_date.to_string());
        domain.auto_renew = Set(auto_renew);
        domain.transfer_lock = Set(transfer_lock);
        domain.auth_code = Set(auth_code);
        domain.whois_privacy = Set(whois_privacy);
        domain.registry_domain_id = Set(registry_domain_id);
        domain.updated_at = Set(chrono::Utc::now().to_rfc3339());

        return domain.update(db).await;
    }

    // Create new domain
    let new_domain = domains::ActiveModel {
        customer_id: Set(customer_id),
        domain_name: Set(domain_name.to_string()),
        tld: Set(tld.to_string()),
        status: Set(status.to_string()),
        registration_date: Set(registration_date.to_string()),
        expiration_date: Set(expiration_date.to_string()),
        auto_renew: Set(auto_renew),
        transfer_lock: Set(transfer_lock),
        auth_code: Set(auth_code),
        whois_privacy: Set(whois_privacy),
        registry_domain_id: Set(registry_domain_id),
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
    domain_name.split('.').last().unwrap_or("").to_string()
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
