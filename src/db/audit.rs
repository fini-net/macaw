use crate::entities::audit_log;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, Set};

/// Log an audit entry
pub async fn log_audit(
    db: &DatabaseConnection,
    table_name: &str,
    record_id: i32,
    action: &str,
    changed_by: &str,
    old_values: Option<String>,
    new_values: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<(), DbErr> {
    let audit_entry = audit_log::ActiveModel {
        table_name: Set(table_name.to_string()),
        record_id: Set(record_id),
        action: Set(action.to_string()),
        changed_by: Set(changed_by.to_string()),
        old_values: Set(old_values),
        new_values: Set(new_values),
        ip_address: Set(ip_address),
        user_agent: Set(user_agent),
        timestamp: Set(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };

    audit_entry.insert(db).await?;
    Ok(())
}

/// Log a domain creation
pub async fn log_domain_creation(
    db: &DatabaseConnection,
    domain_id: i32,
    domain_name: &str,
) -> Result<(), DbErr> {
    log_audit(
        db,
        "domains",
        domain_id,
        "INSERT",
        "sync_process",
        None,
        Some(format!("{{\"domain_name\":\"{}\"}}", domain_name)),
        None,
        None,
    )
    .await
}

/// Log a contact creation
pub async fn log_contact_creation(
    db: &DatabaseConnection,
    contact_id: i32,
    email: &str,
) -> Result<(), DbErr> {
    log_audit(
        db,
        "contacts",
        contact_id,
        "INSERT",
        "sync_process",
        None,
        Some(format!("{{\"email\":\"{}\"}}", email)),
        None,
        None,
    )
    .await
}

/// Log a customer creation
pub async fn log_customer_creation(
    db: &DatabaseConnection,
    customer_id: i32,
    username: &str,
) -> Result<(), DbErr> {
    log_audit(
        db,
        "customers",
        customer_id,
        "INSERT",
        "sync_process",
        None,
        Some(format!("{{\"username\":\"{}\"}}", username)),
        None,
        None,
    )
    .await
}
