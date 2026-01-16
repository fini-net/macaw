use chrono::NaiveDate;
use macaw::config::OpenSrsCredentials;
use macaw::{ClientConfig, Environment, OpenSrsClient};
use std::env;

fn main() {
    println!("Macaw domain registration backend");
    println!();

    match OpenSrsCredentials::from_env() {
        Ok(creds) => {
            println!("✓ OpenSRS credentials loaded successfully");
            println!("  Username: {}", creds.username);

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

            // Test domain listing for 2026
            println!();
            println!("Fetching domains expiring in 2026...");

            let from = NaiveDate::from_ymd_opt(2026, 1, 1).expect("Invalid from date");
            let to = NaiveDate::from_ymd_opt(2026, 12, 31).expect("Invalid to date");

            match client.get_domains_by_expiredate(from, to) {
                Ok(domains) => {
                    println!();
                    println!("Found {} domains expiring in 2026:", domains.len());
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
        Err(e) => {
            eprintln!("✗ Could not load OpenSRS credentials: {}", e);
            eprintln!();
            eprintln!("This is normal if you haven't run with credentials.");
            eprintln!("To run with credentials: just run_with_creds");
            eprintln!();
            eprintln!(
                "To test with production: OPENSRS_ENVIRONMENT=production just run_with_creds"
            );
        }
    }
}
