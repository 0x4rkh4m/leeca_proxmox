use leeca_proxmox::{ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    // Build client with  Proxmox server details
    let mut client = ProxmoxClient::builder()
        .host("192.168.1.182")
        .port(8006)
        .credentials("leeca", "password", "pam")
        .secure(false) // HTTP for local development
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
            "✅ Authenticated"
        } else {
            "❌ Not authenticated"
        }
    );

    println!("\n📡 Connecting to Proxmox...");
    client.login().await?;
    println!(
        "Connection state: {}",
        if client.is_authenticated().await {
            "✅ Authenticated"
        } else {
            "❌ Failed"
        }
    );

    if let Some(token) = client.auth_token().await {
        println!("\n🎟️  Session Token");
        println!("------------------------");
        println!("Value: {}", token.as_str());
    }

    if let Some(csrf) = client.csrf_token().await {
        println!("\n🛡️  CSRF Protection");
        println!("------------------------");
        println!("Token: {}", csrf.as_str());
    }

    println!("\n✨ Connection established successfully!\n");
    Ok(())
}
