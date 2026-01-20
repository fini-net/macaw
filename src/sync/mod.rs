use crate::db;
use crate::opensrs::OpenSrsClient;
use sea_orm::DatabaseConnection;
use std::path::PathBuf;

pub mod config;
pub mod errors;

use config::DomainsConfig;
use errors::{Result, SyncError};

/// Options for the sync operation
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Path to domains.toml configuration
    pub config_path: PathBuf,
    /// Path to database file
    pub database_path: PathBuf,
    /// Specific customer to sync (None = all customers)
    pub customer_filter: Option<String>,
    /// Dry run mode (don't make changes)
    pub dry_run: bool,
}

/// Sync statistics
#[derive(Debug, Default)]
pub struct SyncStats {
    pub customers_created: u32,
    pub domains_synced: u32,
    pub domains_failed: u32,
    pub contacts_created: u32,
    pub nameservers_created: u32,
}

/// Run the sync operation
pub async fn run_sync(client: OpenSrsClient, options: SyncOptions) -> Result<SyncStats> {
    // Load configuration
    let config = DomainsConfig::from_file(&options.config_path)?;

    // Connect to database
    let db = db::connect(options.database_path.to_str().unwrap()).await?;

    let mut stats = SyncStats::default();

    // Determine which customers to sync
    let customers = if let Some(customer) = &options.customer_filter {
        vec![customer.clone()]
    } else {
        config.customers()
    };

    println!("Starting sync for {} customer(s)", customers.len());

    for customer_slug in customers {
        println!("\n=== Syncing customer: {} ===", customer_slug);

        let domains = match config.get_customer_domains(&customer_slug) {
            Some(d) => d,
            None => {
                eprintln!("Warning: No domains found for customer {}", customer_slug);
                continue;
            }
        };

        // Sync this customer
        match sync_customer(&db, &client, &customer_slug, domains, options.dry_run).await {
            Ok(customer_stats) => {
                stats.customers_created += 1;
                stats.domains_synced += customer_stats.domains_synced;
                stats.contacts_created += customer_stats.contacts_created;
                stats.nameservers_created += customer_stats.nameservers_created;
                stats.domains_failed += customer_stats.domains_failed;
            }
            Err(e) => {
                eprintln!("Error syncing customer {}: {}", customer_slug, e);
            }
        }
    }

    println!("\n=== Sync Complete ===");
    println!("Customers created: {}", stats.customers_created);
    println!("Domains synced: {}", stats.domains_synced);
    println!("Domains failed: {}", stats.domains_failed);
    println!("Contacts created: {}", stats.contacts_created);
    println!("Nameservers created: {}", stats.nameservers_created);

    Ok(stats)
}

/// Sync a single customer with their domains
async fn sync_customer(
    db: &DatabaseConnection,
    client: &OpenSrsClient,
    customer_slug: &str,
    domains: &[String],
    dry_run: bool,
) -> Result<SyncStats> {
    let mut stats = SyncStats::default();

    // Use first domain's owner contact email as customer email (placeholder)
    let customer_email = format!("{}@example.com", customer_slug);

    // Create or get customer
    let customer = if dry_run {
        println!("  [DRY RUN] Would create customer: {}", customer_slug);
        // In dry run, just use a fake customer ID
        crate::entities::customers::Model {
            customer_id: 1,
            username: customer_slug.to_string(),
            email: customer_email.clone(),
            company_name: None,
            account_balance: "0.00".parse().unwrap(),
            credit_limit: "0.00".parse().unwrap(),
            status: "active".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    } else {
        let customer =
            db::customers::create_or_get_customer(db, customer_slug, &customer_email).await?;

        // Log audit entry
        db::audit::log_customer_creation(db, customer.customer_id, customer_slug).await?;

        customer
    };

    let customer_id = customer.customer_id;

    println!("  Customer ID: {}", customer_id);
    println!("  Syncing {} domains...", domains.len());

    // Sync each domain
    for domain_name in domains {
        print!("    {}: ", domain_name);

        match sync_domain(db, client, customer_id, domain_name, dry_run).await {
            Ok(domain_stats) => {
                stats.domains_synced += 1;
                stats.contacts_created += domain_stats.contacts_created;
                stats.nameservers_created += domain_stats.nameservers_created;
                println!("✓");
            }
            Err(e) => {
                stats.domains_failed += 1;
                println!("✗ {}", e);
            }
        }
    }

    Ok(stats)
}

/// Sync a single domain
async fn sync_domain(
    db: &DatabaseConnection,
    client: &OpenSrsClient,
    customer_id: i32,
    domain_name: &str,
    dry_run: bool,
) -> Result<SyncStats> {
    let mut stats = SyncStats::default();

    // Fetch domain info from OpenSRS
    // Note: The client's get_domain_all_info is blocking (uses ureq)
    // In a production system, we'd use spawn_blocking, but for simplicity
    // we'll call it directly since the outer function is already async
    let domain_info =
        client
            .get_domain_all_info(domain_name)
            .map_err(|e| SyncError::DomainSyncFailed {
                domain: domain_name.to_string(),
                reason: format!("API error: {}", e),
            })?;

    if dry_run {
        return Ok(stats);
    }

    // Extract TLD
    let tld = db::domains::extract_tld(domain_name);

    // Map status
    let status = db::domains::map_opensrs_status(&domain_info.auto_renew);

    // Parse boolean fields
    let auto_renew = domain_info.auto_renew.to_lowercase() == "y"
        || domain_info.auto_renew.to_lowercase() == "1"
        || domain_info.auto_renew.to_lowercase() == "true";

    let transfer_lock = domain_info.lock_state.to_lowercase() == "1"
        || domain_info.lock_state.to_lowercase() == "locked";

    let whois_privacy = domain_info.whois_privacy_state.to_lowercase() == "enabled"
        || domain_info.whois_privacy_state.to_lowercase() == "1";

    // Create or update domain
    let domain = db::domains::create_or_update_domain(
        db,
        db::domains::DomainData {
            customer_id,
            domain_name: domain_name.to_string(),
            tld,
            status: status.to_string(),
            registration_date: domain_info.registry_createdate.clone(),
            expiration_date: domain_info.domain_expdate.clone(),
            auto_renew,
            transfer_lock,
            auth_code: None, // auth_code - would need separate API call
            whois_privacy,
            registry_domain_id: Some(domain_info.registry_domainid.clone()),
        },
    )
    .await?;

    let domain_id = domain.domain_id;

    // Log audit entry
    db::audit::log_domain_creation(db, domain_id, domain_name).await?;

    // Sync contacts
    let contact_roles = vec![
        ("owner", &domain_info.contact_set.owner),
        ("admin", &domain_info.contact_set.admin),
        ("tech", &domain_info.contact_set.tech),
        ("billing", &domain_info.contact_set.billing),
    ];

    for (role, contact_opt) in contact_roles {
        if let Some(contact) = contact_opt {
            let contact_info = db::contacts::ContactInfo {
                contact_type: role.to_string(),
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
            };

            let contact_id =
                db::contacts::find_or_create_contact(db, customer_id, &contact_info).await?;
            db::contacts::link_contact_to_domain(db, domain_id, contact_id, role).await?;

            stats.contacts_created += 1;

            // Log audit entry
            db::audit::log_contact_creation(db, contact_id, &contact_info.email).await?;
        }
    }

    // Sync nameservers
    for ns in &domain_info.nameserver_list {
        db::nameservers::create_or_update_nameserver(
            db,
            domain_id,
            &ns.name,
            ns.ipaddress.clone(),
            ns.sortorder,
        )
        .await?;

        stats.nameservers_created += 1;
    }

    // Sync TLD-specific data
    for (key, value) in &domain_info.tld_data.data {
        db::tld_data::create_or_update_tld_data(db, domain_id, key, value).await?;
    }

    Ok(stats)
}
