use crate::{
    auth::application::{
        request::login_request::LoginRequest, response::login_response::LoginResponse,
    },
    core::domain::value_object::base_value_object::ValueObject,
    ProxmoxAuth, ProxmoxCSRFToken, ProxmoxConnection, ProxmoxError, ProxmoxResult, ProxmoxTicket,
    ValidationError,
};

use reqwest::{
    header::{HeaderMap, ACCEPT, CONTENT_TYPE},
    Client, StatusCode,
};
use std::backtrace::Backtrace;

pub struct LoginService {
    default_headers: HeaderMap,
}

impl LoginService {
    pub fn new() -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        default_headers.insert(ACCEPT, "application/json".parse().unwrap());

        Self { default_headers }
    }

    pub async fn execute(&self, connection: &ProxmoxConnection) -> ProxmoxResult<ProxmoxAuth> {
        let http_client = Client::builder()
            .danger_accept_invalid_certs(connection.accepts_invalid_certs())
            .build()
            .map_err(|e| ProxmoxError::Connection(e.to_string()))?;
        let url = self.build_login_url(connection).await?;
        let request = self.build_login_request(connection).await?;
        let response = self.send_request(&http_client, &url, &request).await?;

        match response.status() {
            StatusCode::OK => self.handle_successful_login(response).await,
            StatusCode::UNAUTHORIZED => Err(ProxmoxError::Authentication(
                "Invalid credentials provided".to_string(),
            )),
            StatusCode::BAD_REQUEST => Err(ProxmoxError::Validation {
                source: ValidationError::Field {
                    field: "request".to_string(),
                    message: "Invalid request format".to_string(),
                },
                backtrace: Backtrace::capture(),
            }),
            StatusCode::NOT_FOUND => Err(ProxmoxError::Connection(
                "Login endpoint not found".to_string(),
            )),
            StatusCode::SERVICE_UNAVAILABLE => Err(ProxmoxError::Connection(
                "Proxmox service is currently unavailable".to_string(),
            )),
            status => Err(ProxmoxError::Connection(format!(
                "Unexpected response status: {}",
                status
            ))),
        }
    }

    async fn build_login_url(&self, connection: &ProxmoxConnection) -> ProxmoxResult<String> {
        let url = connection
            .proxmox_url()
            .with_path("/api2/json/access/ticket")
            .await?
            .as_inner()
            .await;
        Ok(url)
    }

    async fn build_login_request(
        &self,
        connection: &ProxmoxConnection,
    ) -> ProxmoxResult<LoginRequest> {
        Ok(LoginRequest {
            username: connection.proxmox_username().as_inner().await,
            password: connection.proxmox_password().as_inner().await,
            realm: connection.proxmox_realm().as_inner().await,
        })
    }

    async fn send_request(
        &self,
        client: &Client,
        url: &str,
        request: &LoginRequest,
    ) -> ProxmoxResult<reqwest::Response> {
        client
            .post(url)
            .headers(self.default_headers.clone())
            .json(request)
            .send()
            .await
            .map_err(|e| ProxmoxError::Connection(e.to_string()))
    }

    async fn handle_successful_login(
        &self,
        response: reqwest::Response,
    ) -> ProxmoxResult<ProxmoxAuth> {
        let login_response = response.json::<LoginResponse>().await.map_err(|e| {
            ProxmoxError::Connection(format!("Failed to parse login response: {}", e))
        })?;

        let ticket = ProxmoxTicket::new(login_response.data.ticket).await?;
        let csrf_token = ProxmoxCSRFToken::new(login_response.data.csrf_token).await?;

        ProxmoxAuth::new(ticket, Some(csrf_token)).await
    }
}

impl Default for LoginService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm, ProxmoxUsername};
    use dotenvy::dotenv;
    use std::env;
    use tokio::time::{timeout, Duration};

    async fn setup_connection() -> ProxmoxResult<ProxmoxConnection> {
        dotenv().ok();

        ProxmoxConnection::new(
            ProxmoxHost::new(env::var("PROXMOX_HOST").unwrap()).await?,
            ProxmoxPort::new(env::var("PROXMOX_PORT").unwrap().parse().unwrap()).await?,
            ProxmoxUsername::new(env::var("PROXMOX_USERNAME").unwrap()).await?,
            ProxmoxPassword::new(env::var("PROXMOX_PASSWORD").unwrap()).await?,
            ProxmoxRealm::new(env::var("PROXMOX_REALM").unwrap()).await?,
            false,
            true,
        )
        .await
    }

    #[tokio::test]
    async fn test_login_success() {
        if !has_proxmox_config() {
            println!("Skipping integration test - no Proxmox configuration");
            return;
        }

        let connection = setup_connection().await.unwrap();
        let service = LoginService::new();

        let result = service.execute(&connection).await;
        assert!(result.is_ok());

        let auth = result.unwrap();
        assert!(auth.ticket().value().await.starts_with("PVE:"));
        assert!(auth.csrf_token().is_some());
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        if !has_proxmox_config() {
            println!("Skipping integration test - no Proxmox configuration");
            return;
        }

        let mut connection = setup_connection().await.unwrap();
        // Override with invalid password
        connection = ProxmoxConnection::new(
            connection.proxmox_host().clone(),
            connection.proxmox_port().clone(),
            connection.proxmox_username().clone(),
            ProxmoxPassword::new("InvalidPassword123!".to_string())
                .await
                .unwrap(),
            connection.proxmox_realm().clone(),
            false,
            true,
        )
        .await
        .unwrap();

        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::Authentication(_))));
    }

    #[tokio::test]
    async fn test_login_invalid_endpoint() {
        if !has_proxmox_config() {
            println!("Skipping integration test - no Proxmox configuration");
            return;
        }

        let connection = setup_connection().await.unwrap();
        let service = LoginService::new();

        let invalid_connection = ProxmoxConnection::new(
            ProxmoxHost::new("1.1.1.1".to_string()).await.unwrap(),
            connection.proxmox_port().clone(),
            connection.proxmox_username().clone(),
            connection.proxmox_password().clone(),
            connection.proxmox_realm().clone(),
            false,
            true,
        )
        .await
        .unwrap();

        // Wrap the service execution with a 5-second timeout
        // for the case where the endpoint is unreachable
        let result = timeout(Duration::from_secs(5), service.execute(&invalid_connection)).await;

        assert!(match result {
            Ok(Err(ProxmoxError::Connection(_))) => true,
            Err(_elapsed) => true,
            _ => false,
        });
    }

    // Temporal workaround until github actions secrets are available
    // and running remote Proxmox VE for ci testing
    fn has_proxmox_config() -> bool {
        env::var("PROXMOX_HOST").is_ok()
            && env::var("PROXMOX_PORT").is_ok()
            && env::var("PROXMOX_USERNAME").is_ok()
            && env::var("PROXMOX_PASSWORD").is_ok()
            && env::var("PROXMOX_REALM").is_ok()
    }
}
