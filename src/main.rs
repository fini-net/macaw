use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use macaw::config::OpenSrsCredentials;
use macaw::db::{self, contacts::UpdateStrategy};
use macaw::opensrs::{ContactInfo as OpenSrsContactInfo, ContactInfoForUpdate, SetContactSet};
use macaw::sync::{SyncOptions, run_sync};
use macaw::{ClientConfig, Environment, OpenSrsClient};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "macaw")]
#[command(about = "Macaw domain registration backend", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync domains from OpenSRS to database
    Sync {
        /// Specific customer to sync (default: all)
        #[arg(long)]
        customer: Option<String>,

        /// Dry run - show what would be synced without making changes
        #[arg(long)]
        dry_run: bool,

        /// Path to domains.toml configuration
        #[arg(long, default_value = "domains.toml")]
        config: PathBuf,

        /// Database path
        #[arg(long, default_value = "macaw.db")]
        database: PathBuf,
    },

    /// List domains by expiration year
    List {
        /// Year to query
        year: i32,
    },

    /// Update contact information for a domain
    Contacts {
        /// Domain name to update contacts for
        domain: String,

        /// Contact type to update (owner, admin, tech, billing)
        #[arg(long)]
        contact_type: String,

        /// First name
        #[arg(long)]
        first_name: String,

        /// Last name
        #[arg(long)]
        last_name: String,

        /// Organization name
        #[arg(long)]
        organization: Option<String>,

        /// Email address
        #[arg(long)]
        email: String,

        /// Phone number
        #[arg(long)]
        phone: String,

        /// Fax number
        #[arg(long)]
        fax: Option<String>,

        /// Address line 1
        #[arg(long)]
        address1: String,

        /// Address line 2
        #[arg(long)]
        address2: Option<String>,

        /// City
        #[arg(long)]
        city: String,

        /// State/Province
        #[arg(long)]
        state: String,

        /// Postal code
        #[arg(long)]
        postal_code: String,

        /// Country code
        #[arg(long)]
        country: String,

        /// Database path
        #[arg(long, default_value = "macaw.db")]
        database: PathBuf,

        /// Create new contact instead of updating shared contact
        #[arg(long)]
        create_new: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("Macaw domain registration backend");
    println!();

    // Load OpenSRS credentials
    let creds = match OpenSrsCredentials::from_env() {
        Ok(creds) => {
            println!("✓ OpenSRS credentials loaded successfully");
            println!("  Username: {}", creds.username);
            creds
        }
        Err(e) => {
            eprintln!("✗ Could not load OpenSRS credentials: {}", e);
            eprintln!();
            eprintln!("This is normal if you haven't run with credentials.");
            eprintln!("To run with credentials: just run_with_creds");
            eprintln!();
            eprintln!(
                "To test with production: OPENSRS_ENVIRONMENT=production just run_with_creds"
            );
            std::process::exit(1);
        }
    };

    // Determine environment (default to test for safety)
    let environment = match env::var("OPENSRS_ENVIRONMENT") {
        Ok(val) if val.eq_ignore_ascii_case("production") => {
            println!("  Environment: Production");
            Environment::Production
        }
        _ => {
            println!("  Environment: Test (OT&E)");
            Environment::Test
        }
    };

    // Initialize OpenSRS client
    let config = ClientConfig {
        username: creds.username,
        credential: creds.credential,
        environment,
    };

    let client = OpenSrsClient::new(config);

    // Execute command
    match cli.command {
        Commands::Sync {
            customer,
            dry_run,
            config,
            database,
        } => {
            println!();
            if dry_run {
                println!("=== DRY RUN MODE ===");
                println!();
            }

            let options = SyncOptions {
                config_path: config,
                database_path: database,
                customer_filter: customer,
                dry_run,
            };

            match run_sync(client, options).await {
                Ok(stats) => {
                    println!();
                    println!("Sync completed successfully!");
                    println!("  Customers: {}", stats.customers_created);
                    println!("  Domains synced: {}", stats.domains_synced);
                    println!("  Domains failed: {}", stats.domains_failed);
                }
                Err(e) => {
                    eprintln!();
                    eprintln!("✗ Sync failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::List { year } => {
            println!();
            println!("Fetching domains expiring in {}...", year);

            let from = NaiveDate::from_ymd_opt(year, 1, 1).expect("Invalid from date");
            let to = NaiveDate::from_ymd_opt(year, 12, 31).expect("Invalid to date");

            match client.get_domains_by_expiredate(from, to) {
                Ok(domains) => {
                    println!();
                    println!("Found {} domains expiring in {}:", domains.len(), year);
                    println!();

                    if domains.is_empty() {
                        println!("  (no domains found)");
                    } else {
                        // Show first 10 domains
                        for domain in domains.iter().take(10) {
                            println!(
                                "  {} - expires {} (auto-renew: {})",
                                domain.name, domain.expiredate, domain.f_auto_renew
                            );
                        }

                        if domains.len() > 10 {
                            println!("  ... and {} more", domains.len() - 10);
                        }
                    }
                }
                Err(e) => {
                    eprintln!();
                    eprintln!("✗ Error fetching domains: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Contacts {
            domain,
            contact_type,
            first_name,
            last_name,
            organization,
            email,
            phone,
            fax,
            address1,
            address2,
            city,
            state,
            postal_code,
            country,
            database,
            create_new,
        } => {
            println!();
            println!("Updating {} contact for domain: {}", contact_type, domain);

            // Connect to database
            let db = match db::connect(database.to_str().unwrap()).await {
                Ok(db) => db,
                Err(e) => {
                    eprintln!("✗ Failed to connect to database: {}", e);
                    std::process::exit(1);
                }
            };

            // Get current domain info from OpenSRS
            let domain_info = match client.get_domain_all_info(&domain) {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("✗ Failed to get domain info from OpenSRS: {}", e);
                    std::process::exit(1);
                }
            };

            // Find the contact to update based on contact_type
            let current_contact = match contact_type.as_str() {
                "owner" => domain_info.contact_set.owner,
                "admin" => domain_info.contact_set.admin,
                "tech" => domain_info.contact_set.tech,
                "billing" => domain_info.contact_set.billing,
                _ => {
                    eprintln!("✗ Invalid contact type: {}", contact_type);
                    eprintln!("  Valid types: owner, admin, tech, billing");
                    std::process::exit(1);
                }
            };

            let current_contact = match current_contact {
                Some(c) => c,
                None => {
                    eprintln!("✗ No {} contact found for domain {}", contact_type, domain);
                    std::process::exit(1);
                }
            };

            // Build contact info for update
            let contact_info = db::contacts::ContactInfo {
                contact_type: contact_type.clone(),
                first_name,
                last_name,
                organization,
                email: email.clone(),
                phone,
                fax,
                address1,
                address2,
                city,
                state_province: state,
                postal_code,
                country_code: country,
            };

            // Find domain in database to get customer_id
            let domain_model = macaw::entities::domains::Entity::find()
                .filter(macaw::entities::domains::Column::DomainName.eq(&domain))
                .one(&db)
                .await
                .expect("Database query failed");

            let (customer_id, domain_id) = match domain_model {
                Some(d) => (d.customer_id, d.domain_id),
                None => {
                    eprintln!("✗ Domain {} not found in local database", domain);
                    eprintln!("  Run sync first to import domain data");
                    std::process::exit(1);
                }
            };

            // Find existing contact in database (we need to find by matching email)
            let existing_contact = macaw::entities::contacts::Entity::find()
                .filter(macaw::entities::contacts::Column::Email.eq(&current_contact.email))
                .filter(macaw::entities::contacts::Column::CustomerId.eq(customer_id))
                .one(&db)
                .await
                .expect("Database query failed");

            let existing_contact_id = match existing_contact {
                Some(c) => c.contact_id,
                None => {
                    eprintln!("✗ No matching contact found in local database");
                    std::process::exit(1);
                }
            };

            // Get old contact values for audit
            let old_contact = db::contacts::get_contact(&db, existing_contact_id)
                .await
                .expect("Database query failed")
                .map(|c| db::contacts::ContactInfo {
                    contact_type: c.contact_type,
                    first_name: c.first_name,
                    last_name: c.last_name,
                    organization: c.organization,
                    email: c.email,
                    phone: c.phone,
                    fax: c.fax,
                    address1: c.address1,
                    address2: c.address2,
                    city: c.city,
                    state_province: c.state_province,
                    postal_code: c.postal_code,
                    country_code: c.country_code,
                });

            // Update contact in OpenSRS first
            let mut set_contact = SetContactSet::default();
            let openrs_contact = ContactInfoForUpdate::from(&OpenSrsContactInfo {
                first_name: contact_info.first_name.clone(),
                last_name: contact_info.last_name.clone(),
                org_name: contact_info.organization.clone(),
                email: contact_info.email.clone(),
                phone: contact_info.phone.clone(),
                fax: contact_info.fax.clone(),
                address1: contact_info.address1.clone(),
                address2: contact_info.address2.clone(),
                city: contact_info.city.clone(),
                state: contact_info.state_province.clone(),
                postal_code: contact_info.postal_code.clone(),
                country: contact_info.country_code.clone(),
            });

            match contact_type.as_str() {
                "owner" => set_contact.owner = Some(openrs_contact),
                "admin" => set_contact.admin = Some(openrs_contact),
                "tech" => set_contact.tech = Some(openrs_contact),
                "billing" => set_contact.billing = Some(openrs_contact),
                _ => {}
            }

            println!("  Sending update to OpenSRS...");

            let response = match client.update_domain_contacts(&domain, set_contact) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("✗ Failed to update contacts in OpenSRS: {}", e);
                    std::process::exit(1);
                }
            };

            if !response.is_success {
                eprintln!(
                    "✗ OpenSRS returned error: {} - {}",
                    response.response_code, response.response_text
                );
                std::process::exit(1);
            }

            println!("  ✓ OpenSRS updated successfully");

            // Now update local database
            let strategy = if create_new {
                UpdateStrategy::CreateNew
            } else {
                UpdateStrategy::UpdateInPlace
            };

            let new_contact_id = match db::contacts::update_contact(
                &db,
                customer_id,
                existing_contact_id,
                &contact_info,
                strategy,
                Some(domain_id),
            )
            .await
            {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("✗ Failed to update local contact: {}", e);
                    std::process::exit(1);
                }
            };

            println!("  ✓ Local database updated");

            // Log audit entry
            let new_contact = db::contacts::get_contact(&db, new_contact_id)
                .await
                .expect("Database query failed")
                .map(|c| db::contacts::ContactInfo {
                    contact_type: c.contact_type,
                    first_name: c.first_name,
                    last_name: c.last_name,
                    organization: c.organization,
                    email: c.email,
                    phone: c.phone,
                    fax: c.fax,
                    address1: c.address1,
                    address2: c.address2,
                    city: c.city,
                    state_province: c.state_province,
                    postal_code: c.postal_code,
                    country_code: c.country_code,
                });

            if let Err(e) = db::audit::log_contact_update(
                &db,
                new_contact_id,
                old_contact.as_ref(),
                new_contact.as_ref(),
                "cli_user",
                None,
                None,
            )
            .await
            {
                eprintln!("  ⚠ Failed to write audit log: {}", e);
            } else {
                println!("  ✓ Audit log written");
            }

            println!();
            println!("Contact update completed successfully!");
        }
    }
}
