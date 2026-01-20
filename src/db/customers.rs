use crate::entities::{customers, prelude::Customers};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

/// Create a new customer or return existing one if username already exists
pub async fn create_or_get_customer(
    db: &DatabaseConnection,
    username: &str,
    email: &str,
) -> Result<customers::Model, DbErr> {
    // Check if customer already exists
    if let Some(existing) = Customers::find()
        .filter(customers::Column::Username.eq(username))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    // Create new customer
    let new_customer = customers::ActiveModel {
        username: Set(username.to_string()),
        email: Set(email.to_string()),
        company_name: Set(None),
        account_balance: Set("0.00".parse().unwrap()),
        credit_limit: Set("0.00".parse().unwrap()),
        status: Set("active".to_string()),
        created_at: Set(chrono::Utc::now().to_rfc3339()),
        updated_at: Set(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };

    let customer = new_customer.insert(db).await?;
    Ok(customer)
}

/// Get customer by username
pub async fn get_customer_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<customers::Model>, DbErr> {
    Customers::find()
        .filter(customers::Column::Username.eq(username))
        .one(db)
        .await
}

/// Get customer by ID
pub async fn get_customer_by_id(
    db: &DatabaseConnection,
    customer_id: i32,
) -> Result<Option<customers::Model>, DbErr> {
    Customers::find_by_id(customer_id).one(db).await
}

/// Get customer ID from model
pub fn get_customer_id(customer: &customers::Model) -> i32 {
    customer.customer_id
}
