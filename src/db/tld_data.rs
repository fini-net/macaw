use crate::entities::{prelude::TldData, tld_data};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

/// Create or update TLD-specific data for a domain
pub async fn create_or_update_tld_data(
    db: &DatabaseConnection,
    domain_id: i32,
    data_key: &str,
    data_value: &str,
) -> Result<tld_data::Model, DbErr> {
    // Check if this key already exists for this domain
    if let Some(existing) = TldData::find()
        .filter(tld_data::Column::DomainId.eq(domain_id))
        .filter(tld_data::Column::DataKey.eq(data_key))
        .one(db)
        .await?
    {
        // Update existing data
        let mut data: tld_data::ActiveModel = existing.into();
        data.data_value = Set(data_value.to_string());
        data.updated_at = Set(chrono::Utc::now().to_rfc3339());

        return data.update(db).await;
    }

    // Create new TLD data
    let new_data = tld_data::ActiveModel {
        domain_id: Set(domain_id),
        data_key: Set(data_key.to_string()),
        data_value: Set(data_value.to_string()),
        created_at: Set(chrono::Utc::now().to_rfc3339()),
        updated_at: Set(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };

    new_data.insert(db).await
}

/// Get TLD-specific data for a domain
pub async fn get_tld_data(
    db: &DatabaseConnection,
    domain_id: i32,
) -> Result<Vec<tld_data::Model>, DbErr> {
    TldData::find()
        .filter(tld_data::Column::DomainId.eq(domain_id))
        .all(db)
        .await
}
