use leeca_proxmox::{ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let mut client = ProxmoxClient::builder()
        .host("proxmox.example.com")?
        .port(8006)?
        .credentials("admin", "password", "pve")?
        .secure(true)
        .build()
        .await?;

    client.login().await?;
    Ok(())
}
