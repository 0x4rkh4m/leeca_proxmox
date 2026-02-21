use crate::{ProxmoxClient, ProxmoxResult};
use dotenvy::dotenv;
use std::env;
use tempfile::NamedTempFile;

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
    assert!(client.is_authenticated().await);
    assert!(client.auth_token().await.is_some());
    assert!(client.csrf_token().await.is_some());

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

#[tokio::test]
#[ignore = "requires running Proxmox instance and environment variables"]
async fn test_integration_session_persistence() -> ProxmoxResult<()> {
    setup();
    let host = env::var("PROXMOX_HOST").expect("PROXMOX_HOST not set");
    let port: u16 = env::var("PROXMOX_PORT")
        .expect("PROXMOX_PORT not set")
        .parse()
        .expect("invalid port");
    let username = env::var("PROXMOX_USERNAME").expect("PROXMOX_USERNAME not set");
    let password = env::var("PROXMOX_PASSWORD").expect("PROXMOX_PASSWORD not set");
    let realm = env::var("PROXMOX_REALM").expect("PROXMOX_REALM not set");

    // Create client and login
    let mut client = ProxmoxClient::builder()
        .host(&host)
        .port(port)
        .credentials(username.clone(), password.clone(), realm.clone())
        .secure(true)
        .accept_invalid_certs(true)
        .build()
        .await?;

    client.login().await?;
    assert!(client.is_authenticated().await);

    // Save session to a temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    client.save_session_to_file(&path).await?;

    // Create a new client with the same connection settings and load the session
    let new_client = ProxmoxClient::builder()
        .host(host)
        .port(port)
        .credentials(username, password, realm)
        .secure(true)
        .accept_invalid_certs(true)
        .with_session(std::fs::File::open(&path)?)
        .await?
        .build()
        .await?;

    // Should be authenticated without calling login
    assert!(new_client.is_authenticated().await);
    assert_eq!(
        client.auth_token().await.unwrap().as_str(),
        new_client.auth_token().await.unwrap().as_str()
    );

    Ok(())
}
