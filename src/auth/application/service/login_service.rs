use crate::{
    ProxmoxAuth, ProxmoxCSRFToken, ProxmoxConnection, ProxmoxError, ProxmoxResult, ProxmoxTicket,
    ValidationError,
    auth::application::{
        request::login_request::LoginRequest, response::login_response::LoginResponse,
    },
};

use reqwest::{
    Client, StatusCode,
    header::{ACCEPT, CONTENT_TYPE, HeaderMap},
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

        // Add Cloudflare Access headers if environment variables are present
        if let Ok(client_id) = std::env::var("CF_ACCESS_CLIENT_ID")
            && !client_id.is_empty()
        {
            default_headers.insert(
                "CF-Access-Client-Id",
                format!("{}.access", client_id).parse().unwrap(),
            );

            if let Ok(client_secret) = std::env::var("CF_ACCESS_CLIENT_SECRET")
                && !client_secret.is_empty()
            {
                default_headers.insert("CF-Access-Client-Secret", client_secret.parse().unwrap());
            }
        }

        Self { default_headers }
    }

    pub async fn execute(&self, connection: &ProxmoxConnection) -> ProxmoxResult<ProxmoxAuth> {
        println!("Building HTTP client with connection settings");
        let http_client = Client::builder()
            .danger_accept_invalid_certs(connection.accept_invalid_certs())
            .build()
            .map_err(|e| {
                println!("Failed to build HTTP client: {}", e);
                ProxmoxError::Connection(e.to_string())
            })?;

        let url = self.build_login_url(connection)?;
        println!("URL built: {}", url);

        let request = self.build_login_request(connection);
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

    fn build_login_url(&self, connection: &ProxmoxConnection) -> ProxmoxResult<String> {
        // We construct the full URL directly (simpler, no need for ProxmoxUrl::with_path)
        let base = connection.url().as_str();
        let base = base.trim_end_matches('/');
        Ok(format!("{}/api2/json/access/ticket", base))
    }

    fn build_login_request(&self, connection: &ProxmoxConnection) -> LoginRequest {
        LoginRequest {
            username: connection.username().as_str().to_string(),
            password: connection.password().as_str().to_string(),
            realm: connection.realm().as_str().to_string(),
        }
    }

    async fn send_request(
        &self,
        client: &Client,
        url: &str,
        request: &LoginRequest,
    ) -> ProxmoxResult<reqwest::Response> {
        println!("Sending request to: {}", url);
        let response = client
            .post(url)
            .headers(self.default_headers.clone())
            .json(request)
            .send()
            .await;

        match response {
            Ok(r) => {
                println!("Response status: {}", r.status());
                println!("Response headers: {:?}", r.headers());
                Ok(r)
            }
            Err(e) => {
                println!("Request failed: {}", e);
                Err(ProxmoxError::Connection(e.to_string()))
            }
        }
    }

    async fn handle_successful_login(
        &self,
        response: reqwest::Response,
    ) -> ProxmoxResult<ProxmoxAuth> {
        let login_response = response.json::<LoginResponse>().await.map_err(|e| {
            ProxmoxError::Connection(format!("Failed to parse login response: {}", e))
        })?;

        // Validate ticket and CSRF token format (optional, but good to catch server errors)
        let ticket_str = login_response.data.ticket;
        let csrf_str = login_response.data.csrf_token;

        // Use validation functions to ensure format (they return errors if invalid)
        crate::core::domain::value_object::validate_ticket(&ticket_str).map_err(|e| {
            ProxmoxError::Validation {
                source: e,
                backtrace: Backtrace::capture(),
            }
        })?;
        crate::core::domain::value_object::validate_csrf_token(&csrf_str).map_err(|e| {
            ProxmoxError::Validation {
                source: e,
                backtrace: Backtrace::capture(),
            }
        })?;

        let ticket = ProxmoxTicket::new_unchecked(ticket_str);
        let csrf_token = ProxmoxCSRFToken::new_unchecked(csrf_str);

        Ok(ProxmoxAuth::new(ticket, Some(csrf_token)))
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
    use crate::{
        ProxmoxClientBuilder, ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm,
        ProxmoxUsername,
    };
    use dotenvy::dotenv;
    use std::env;
    use tokio::time::{Duration, timeout};

    fn setup_env() {
        dotenv().ok();
    }

    async fn create_test_connection() -> ProxmoxResult<ProxmoxConnection> {
        setup_env();

        let host = ProxmoxHost::new_unchecked(env::var("PROXMOX_HOST").unwrap());
        let port = ProxmoxPort::new_unchecked(env::var("PROXMOX_PORT").unwrap().parse().unwrap());
        let username = ProxmoxUsername::new_unchecked(env::var("PROXMOX_USERNAME").unwrap());
        let password = ProxmoxPassword::new_unchecked(env::var("PROXMOX_PASSWORD").unwrap());
        let realm = ProxmoxRealm::new_unchecked(env::var("PROXMOX_REALM").unwrap());

        let url_str = format!("https://{}:{}/", host.as_str(), port.get());
        let url = crate::ProxmoxUrl::new_unchecked(url_str);

        Ok(ProxmoxConnection::new(
            host, port, username, password, realm, true, // secure
            true, // accept_invalid_certs (for self-signed)
            url,
        ))
    }

    #[tokio::test]
    #[ignore = "requires running Proxmox instance"]
    async fn test_login_success() {
        let connection = create_test_connection().await.unwrap();
        let service = LoginService::new();

        let result = service.execute(&connection).await;
        assert!(result.is_ok());

        let auth = result.unwrap();
        assert!(auth.ticket().as_str().starts_with("PVE:"));
        assert!(auth.csrf_token().is_some());
    }

    #[tokio::test]
    #[ignore = "requires running Proxmox instance"]
    async fn test_login_invalid_credentials() {
        let mut connection = create_test_connection().await.unwrap();
        // Override with invalid password
        let invalid_password = ProxmoxPassword::new_unchecked("InvalidPassword123!".to_string());
        connection = ProxmoxConnection::new(
            connection.host().clone(),
            connection.port().clone(),
            connection.username().clone(),
            invalid_password,
            connection.realm().clone(),
            true,
            true,
            connection.url().clone(),
        );

        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::Authentication(_))));
    }

    #[tokio::test]
    #[ignore = "requires running Proxmox instance"]
    async fn test_login_invalid_endpoint() {
        let connection = create_test_connection().await.unwrap();
        let service = LoginService::new();

        let invalid_host = ProxmoxHost::new_unchecked("1.1.1.1".to_string());
        let invalid_url_str = format!(
            "https://{}:{}/",
            invalid_host.as_str(),
            connection.port().get()
        );
        let invalid_url = crate::ProxmoxUrl::new_unchecked(invalid_url_str);

        let invalid_connection = ProxmoxConnection::new(
            invalid_host,
            connection.port().clone(),
            connection.username().clone(),
            connection.password().clone(),
            connection.realm().clone(),
            true,
            true,
            invalid_url,
        );

        let result = timeout(Duration::from_secs(5), service.execute(&invalid_connection)).await;
        assert!(matches!(
            result,
            Ok(Err(ProxmoxError::Connection(_))) | Err(_)
        ));
    }
}
