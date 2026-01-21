use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use macaw::config::OpenSrsCredentials;
use macaw::sync::{SyncOptions, run_sync};
use macaw::{ClientConfig, Environment, OpenSrsClient};
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
    }
}
