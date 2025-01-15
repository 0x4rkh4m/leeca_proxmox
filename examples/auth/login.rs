use leeca_proxmox::{ProxmoxClient, ProxmoxResult};
use std::time::UNIX_EPOCH;

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let mut client = ProxmoxClient::builder()
        .host("192.168.1.182")?
        .port(8006)?
        .credentials("leeca", "Leeca_proxmox1!", "pam")?
        .secure(false)
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
        println!("Value: {}", token.value().await);
        let expires = token
            .expires_at()
            .await
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        println!("Expires at: {} (Unix timestamp)", expires);
    }

    if let Some(csrf) = client.csrf_token() {
        println!("\n🛡️  CSRF Protection");
        println!("------------------------");
        println!("Token: {}", csrf.value().await);
        let expires = csrf
            .expires_at()
            .await
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        println!("Expires at: {} (Unix timestamp)", expires);
    }

    println!("\n✨ Connection established successfully!\n");
    Ok(())
}
