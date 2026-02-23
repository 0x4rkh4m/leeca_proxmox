//! Session persistence workflow for a Proxmox client.
//!
//! This program authenticates against a Proxmox server, stores the session
//! to disk, and then restores it into a new client instance without
//! performing a second login.

use leeca_proxmox::{ProxmoxClient, ProxmoxResult};
use std::fs::File;

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let host = "192.168.1.182";
    let port: u16 = 8006;
    let username = "leeca";
    let password = "password";
    let realm = "pam";

    // Build and authenticate the initial client.
    let mut client = ProxmoxClient::builder()
        .host(host)
        .port(port)
        .credentials(username, password, realm)
        .secure(false) // HTTP for local development
        .accept_invalid_certs(true) // Testing & self-signed certs
        // Optional validation:
        // .enable_password_strength(3)
        // .block_reserved_usernames()
        // ...
        .build()
        .await?;

    client.login().await?;

    println!(
        "Logged in successfully. Ticket: {}",
        client.auth_token().await.unwrap().as_str()
    );

    // Persist the authenticated session to disk asynchronously.
    let session_path = "session.json";
    client.save_session_to_file(session_path).await?;
    println!("Session saved to {}", session_path);

    // Create a new client instance and restore the session.
    let session_file = File::open(session_path)?;
    let new_client = ProxmoxClient::builder()
        .host(host)
        .port(port)
        .credentials(username, password, realm)
        .secure(false)
        .with_session(session_file)
        .await?
        .build()
        .await?;

    println!(
        "Session loaded. Authenticated: {}",
        new_client.is_authenticated().await
    );

    // Ensure both clients share the same authentication token.
    assert_eq!(
        client.auth_token().await.unwrap().as_str(),
        new_client.auth_token().await.unwrap().as_str()
    );

    // Remove the persisted session file.
    std::fs::remove_file(session_path)?;

    Ok(())
}
