use crate::entities::audit_log;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, Set};

/// Audit log entry data
pub struct AuditEntry {
    pub table_name: String,
    pub record_id: i32,
    pub action: String,
    pub changed_by: String,
    pub old_values: Option<String>,
    pub new_values: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Log an audit entry
pub async fn log_audit(db: &DatabaseConnection, entry: AuditEntry) -> Result<(), DbErr> {
    let audit_entry = audit_log::ActiveModel {
        table_name: Set(entry.table_name),
        record_id: Set(entry.record_id),
        action: Set(entry.action),
        changed_by: Set(entry.changed_by),
        old_values: Set(entry.old_values),
        new_values: Set(entry.new_values),
        ip_address: Set(entry.ip_address),
        user_agent: Set(entry.user_agent),
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
        AuditEntry {
            table_name: "domains".to_string(),
            record_id: domain_id,
            action: "INSERT".to_string(),
            changed_by: "sync_process".to_string(),
            old_values: None,
            new_values: Some(format!("{{\"domain_name\":\"{}\"}}", domain_name)),
            ip_address: None,
            user_agent: None,
        },
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
        AuditEntry {
            table_name: "contacts".to_string(),
            record_id: contact_id,
            action: "INSERT".to_string(),
            changed_by: "sync_process".to_string(),
            old_values: None,
            new_values: Some(format!("{{\"email\":\"{}\"}}", email)),
            ip_address: None,
            user_agent: None,
        },
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
        AuditEntry {
            table_name: "customers".to_string(),
            record_id: customer_id,
            action: "INSERT".to_string(),
            changed_by: "sync_process".to_string(),
            old_values: None,
            new_values: Some(format!("{{\"username\":\"{}\"}}", username)),
            ip_address: None,
            user_agent: None,
        },
    )
    .await
}

/// Log a contact update with old and new values as JSON
pub async fn log_contact_update(
    db: &DatabaseConnection,
    contact_id: i32,
    old_values: Option<&crate::db::contacts::ContactInfo>,
    new_values: Option<&crate::db::contacts::ContactInfo>,
    changed_by: &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), DbErr> {
    let old_json = old_values.map(|c| {
        serde_json::to_string(c).unwrap_or_default()
    });
    let new_json = new_values.map(|c| {
        serde_json::to_string(c).unwrap_or_default()
    });

    log_audit(
        db,
        AuditEntry {
            table_name: "contacts".to_string(),
            record_id: contact_id,
            action: "UPDATE".to_string(),
            changed_by: changed_by.to_string(),
            old_values: old_json,
            new_values: new_json,
            ip_address: ip_address.map(String::from),
            user_agent: user_agent.map(String::from),
        },
    )
    .await
}

/// Log a contact creation with full details as JSON
pub async fn log_contact_update_with_details(
    db: &DatabaseConnection,
    contact_id: i32,
    old_values_json: Option<String>,
    new_values_json: Option<String>,
    changed_by: &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), DbErr> {
    log_audit(
        db,
        AuditEntry {
            table_name: "contacts".to_string(),
            record_id: contact_id,
            action: "UPDATE".to_string(),
            changed_by: changed_by.to_string(),
            old_values: old_values_json,
            new_values: new_values_json,
            ip_address: ip_address.map(String::from),
            user_agent: user_agent.map(String::from),
        },
    )
    .await
}
