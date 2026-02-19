use crate::{ProxmoxClient, ProxmoxResult};
use dotenvy::dotenv;
use std::env;

fn setup() {
    dotenv().ok();
}

#[tokio::test]
#[ignore = "requires running Proxmox instance and environment variables"]
async fn test_integration_login_success() -> ProxmoxResult<()> {
    setup();
    let host = env::var("PROXMOX_HOST").expect("PROXMOX_HOST not set");
    let port: u16 = env::var("PROXMOX_PORT")
        .expect("PROXMOX_PORT not set")
        .parse()
        .expect("invalid port");
    let username = env::var("PROXMOX_USERNAME").expect("PROXMOX_USERNAME not set");
    let password = env::var("PROXMOX_PASSWORD").expect("PROXMOX_PASSWORD not set");
    let realm = env::var("PROXMOX_REALM").expect("PROXMOX_REALM not set");

    let mut client = ProxmoxClient::builder()
        .host(host)
        .port(port)
        .credentials(username, password, realm)
        .secure(true)
        .accept_invalid_certs(true) // allow self-signed certs for testing
        .build()
        .await?;

    client.login().await?;
    assert!(client.is_authenticated());
    assert!(client.auth_token().is_some());
    assert!(client.csrf_token().is_some());

    Ok(())
}

#[tokio::test]
#[ignore = "requires running Proxmox instance and environment variables"]
async fn test_integration_login_invalid_credentials() -> ProxmoxResult<()> {
    setup();
    let host = env::var("PROXMOX_HOST").expect("PROXMOX_HOST not set");
    let port: u16 = env::var("PROXMOX_PORT")
        .expect("PROXMOX_PORT not set")
        .parse()
        .expect("invalid port");
    let realm = env::var("PROXMOX_REALM").expect("PROXMOX_REALM not set");

    let mut client = ProxmoxClient::builder()
        .host(host)
        .port(port)
        .credentials("invalid_user", "invalid_pass", realm)
        .secure(true)
        .accept_invalid_certs(true)
        .build()
        .await?;

    let result = client.login().await;
    assert!(result.is_err());
    // Optionally check error type
    // assert!(matches!(result.unwrap_err(), ProxmoxError::Authentication(_)));
    Ok(())
}
