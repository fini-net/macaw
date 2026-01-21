use crate::entities::{nameservers, prelude::Nameservers};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

/// Create or update a nameserver for a domain
pub async fn create_or_update_nameserver(
    db: &DatabaseConnection,
    domain_id: i32,
    hostname: &str,
    ip_address: Option<String>,
    priority: i32,
) -> Result<nameservers::Model, DbErr> {
    // Check if nameserver already exists for this domain and hostname
    if let Some(existing) = Nameservers::find()
        .filter(nameservers::Column::DomainId.eq(domain_id))
        .filter(nameservers::Column::Hostname.eq(hostname))
        .one(db)
        .await?
    {
        // Update existing nameserver
        let mut ns: nameservers::ActiveModel = existing.into();
        ns.ip_address = Set(ip_address);
        ns.priority = Set(priority);

        return ns.update(db).await;
    }

    // Create new nameserver
    let new_ns = nameservers::ActiveModel {
        domain_id: Set(domain_id),
        hostname: Set(hostname.to_string()),
        ip_address: Set(ip_address),
        priority: Set(priority),
        created_at: Set(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };

    new_ns.insert(db).await
}

/// Delete all nameservers for a domain
pub async fn delete_nameservers_for_domain(
    db: &DatabaseConnection,
    domain_id: i32,
) -> Result<(), DbErr> {
    Nameservers::delete_many()
        .filter(nameservers::Column::DomainId.eq(domain_id))
        .exec(db)
        .await?;

    Ok(())
}
