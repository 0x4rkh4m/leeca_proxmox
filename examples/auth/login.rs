//! Basic authentication flow against a Proxmox server.
//!
//! This program builds a `ProxmoxClient`, checks its initial authentication
//! state, performs login, and then retrieves the issued session and CSRF tokens.

use leeca_proxmox::{ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    // Build the client with server configuration and credentials.
    let mut client = ProxmoxClient::builder()
        .host("192.168.1.182")
        .port(8006)
        .credentials("leeca", "password", "pam")
        .secure(false) // HTTP for local development
        .accept_invalid_certs(true) // Testing & Self signed certs
        // Optional validation:
        // .enable_password_strength(3)
        // .block_reserved_usernames()
        // ...
        .build()
        .await?;

    println!("\n🔑 Authentication Status");
    println!("------------------------");
    println!(
        "Initial state: {}",
        if client.is_authenticated().await {
            "Authenticated"
        } else {
            "Not authenticated"
        }
    );

    // Perform authentication against the Proxmox API.
    println!("\n📡 Connecting to Proxmox...");
    client.login().await?;

    println!(
        "Connection state: {}",
        if client.is_authenticated().await {
            "Authenticated"
        } else {
            "Failed"
        }
    );

    // Retrieve the issued authentication ticket.
    if let Some(token) = client.auth_token().await {
        println!("\n🎟️  Session Token");
        println!("------------------------");
        println!("Value: {}", token.as_str());
    }

    // Retrieve the CSRF prevention token used for state-changing operations.
    if let Some(csrf) = client.csrf_token().await {
        println!("\n🛡️  CSRF Token");
        println!("------------------------");
        println!("Value: {}", csrf.as_str());
    }

    println!("\nConnection established successfully.\n");

    Ok(())
}
