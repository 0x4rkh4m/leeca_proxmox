use leeca_proxmox::{ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let mut client = ProxmoxClient::builder()
        .host("192.168.1.182")?
        .port(8006)?
        .credentials("leeca", "Leeca_proxmox1!", "pam")?
        .secure(false)
        .build()
        .await?;

    client.login().await?;
    println!("Authenticated: {}", client.is_authenticated());

    if let Some(token) = client.auth_token() {
        println!("Session Token: {}", token.value().await);
        println!("Session Token expires at: {:?}", token.expires_at().await);
    }

    if let Some(csrf) = client.csrf_token() {
        println!("CSRF Token: {}", csrf.value().await);
        println!("CSRF Token expires at: {:?}", csrf.expires_at().await);
    }

    Ok(())
}
