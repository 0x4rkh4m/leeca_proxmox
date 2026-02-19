use leeca_proxmox::{ProxmoxClient, ProxmoxResult};
use std::time::UNIX_EPOCH;

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let mut client = ProxmoxClient::builder()
        .host("192.168.1.182")
        .port(8006)
        .credentials("leeca", "password", "pam")
        .secure(false)
        // Optional: enable extra checks
        // .enable_password_strength(3)
        // .block_reserved_usernames()
        .build()
        .await?;

    println!("\n🔑 Authentication Status");
    println!("------------------------");
    println!(
        "Initial state: {}",
        if client.is_authenticated() {
            "✅ Authenticated"
        } else {
            "❌ Not authenticated"
        }
    );

    println!("\n📡 Connecting to Proxmox...");
    client.login().await?;
    println!(
        "Connection state: {}",
        if client.is_authenticated() {
            "✅ Authenticated"
        } else {
            "❌ Failed"
        }
    );

    if let Some(token) = client.auth_token() {
        println!("\n🎟️  Session Token");
        println!("------------------------");
        println!("Value: {}", token.as_str());
        let expires = token
            .created_at()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 7200; // approximate
        println!("Expires at: {} (Unix timestamp approx)", expires);
    }

    if let Some(csrf) = client.csrf_token() {
        println!("\n🛡️  CSRF Protection");
        println!("------------------------");
        println!("Token: {}", csrf.as_str());
        let expires = csrf
            .created_at()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 300;
        println!("Expires at: {} (Unix timestamp approx)", expires);
    }

    println!("\n✨ Connection established successfully!\n");
    Ok(())
}
