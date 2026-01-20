use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr};
use std::time::Duration;

pub mod audit;
pub mod contacts;
pub mod customers;
pub mod domains;
pub mod nameservers;
pub mod tld_data;

/// Connect to the SQLite database at the specified path
pub async fn connect(database_path: &str) -> Result<DatabaseConnection, DbErr> {
    let database_url = format!("sqlite://{}?mode=rwc", database_path);

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;

    // Enable foreign keys for SQLite
    db.execute_unprepared("PRAGMA foreign_keys = ON;").await?;

    Ok(db)
}
