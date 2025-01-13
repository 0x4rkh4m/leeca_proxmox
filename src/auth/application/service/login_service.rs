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
    http_client: Client,
    default_headers: HeaderMap,
}

impl LoginService {
    pub fn new() -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        default_headers.insert(ACCEPT, "application/json".parse().unwrap());

        Self {
            http_client: Client::new(),
            default_headers,
        }
    }

    pub async fn execute(&self, connection: &ProxmoxConnection) -> ProxmoxResult<ProxmoxAuth> {
        let url = self.build_login_url(connection).await?;
        let request = self.build_login_request(connection).await?;
        let response = self.send_request(&url, &request).await?;

        match response.status() {
            StatusCode::OK => self.handle_successful_login(response).await,
            StatusCode::UNAUTHORIZED => Err(ProxmoxError::AuthenticationError(
                "Invalid credentials provided".to_string(),
            )),
            StatusCode::BAD_REQUEST => Err(ProxmoxError::ValidationError {
                source: ValidationError::FieldError {
                    field: "request".to_string(),
                    message: "Invalid request format".to_string(),
                },
                backtrace: Backtrace::capture(),
            }),
            StatusCode::NOT_FOUND => Err(ProxmoxError::ConnectionError(
                "Login endpoint not found".to_string(),
            )),
            StatusCode::SERVICE_UNAVAILABLE => Err(ProxmoxError::ConnectionError(
                "Proxmox service is currently unavailable".to_string(),
            )),
            status => Err(ProxmoxError::ConnectionError(format!(
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
        url: &str,
        request: &LoginRequest,
    ) -> ProxmoxResult<reqwest::Response> {
        self.http_client
            .post(url)
            .headers(self.default_headers.clone())
            .json(request)
            .send()
            .await
            .map_err(|e| ProxmoxError::ConnectionError(e.to_string()))
    }

    async fn handle_successful_login(
        &self,
        response: reqwest::Response,
    ) -> ProxmoxResult<ProxmoxAuth> {
        let login_response = response.json::<LoginResponse>().await.map_err(|e| {
            ProxmoxError::ConnectionError(format!("Failed to parse login response: {}", e))
        })?;

        let ticket = ProxmoxTicket::new(login_response.ticket).await?;
        let csrf_token = ProxmoxCSRFToken::new(login_response.csrf_token).await?;

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
    use mockall::predicate::*;
    use serde_json::json;
    use wiremock::{matchers::method, matchers::path, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_login_success() {
        let mock_server = MockServer::start().await;
        let connection = create_test_connection(&mock_server.uri()).await;

        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "ticket": "valid-ticket",
                "csrf_token": "valid-csrf-token"
            })))
            .mount(&mock_server)
            .await;

        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let mock_server = MockServer::start().await;
        let connection = create_test_connection(&mock_server.uri()).await;

        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::AuthenticationError(_))));
    }

    #[tokio::test]
    async fn test_login_server_error() {
        let mock_server = MockServer::start().await;
        let connection = create_test_connection(&mock_server.uri()).await;

        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::ConnectionError(_))));
    }

    async fn create_test_connection(base_url: &str) -> ProxmoxConnection {
        ProxmoxConnection::new(
            ProxmoxHost::new(base_url.to_string()).await.unwrap(),
            ProxmoxPort::new(8006).await.unwrap(),
            ProxmoxUsername::new("test-user".to_string()).await.unwrap(),
            ProxmoxPassword::new("test-pass".to_string()).await.unwrap(),
            ProxmoxRealm::new("pam".to_string()).await.unwrap(),
            true,
        )
        .await
        .unwrap()
    }
}
