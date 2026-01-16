use macaw::config::OpenSrsCredentials;

fn main() {
    println!("Macaw domain registration backend");

    match OpenSrsCredentials::from_env() {
        Ok(creds) => {
            println!("OpenSRS credentials loaded successfully");
            println!("Username: {}", creds.username);
            println!("Credential: {} characters", creds.credential.len());

            // Future: Initialize OpenSRS API client
        }
        Err(e) => {
            eprintln!("Warning: Could not load OpenSRS credentials: {}", e);
            eprintln!("This is normal if you haven't run with credentials.");
            eprintln!("To run with credentials: just run_with_creds");
        }
    }
}
