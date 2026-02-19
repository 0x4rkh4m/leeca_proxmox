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
        ProxmoxConnection, ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm, ProxmoxUrl,
        ProxmoxUsername,
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    async fn create_test_connection(server_url: &str) -> ProxmoxConnection {
        let host = ProxmoxHost::new_unchecked(server_url.trim_start_matches("http://").to_string());
        let port = ProxmoxPort::new_unchecked(8006);
        let username = ProxmoxUsername::new_unchecked("testuser".to_string());
        let password = ProxmoxPassword::new_unchecked("testpass".to_string());
        let realm = ProxmoxRealm::new_unchecked("pam".to_string());
        let url = ProxmoxUrl::new_unchecked(server_url.to_string() + "/");
        ProxmoxConnection::new(host, port, username, password, realm, false, true, url)
    }

    #[tokio::test]
    async fn test_login_service_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "ticket": "PVE:testuser@pam:4EEC61E2::rsKoApxDTLYPn6H3NNT6iP2mv",
                    "CSRFPreventionToken": "4EEC61E2:lwk7od06fa1+DcPUwBTXCcndyAY/3mKxQp5vR8sNjWuBtL9fZg=="
                }
            })))
            .mount(&mock_server)
            .await;

        let connection = create_test_connection(&mock_server.uri()).await;
        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(result.is_ok());
        let auth = result.unwrap();
        assert_eq!(
            auth.ticket().as_str(),
            "PVE:testuser@pam:4EEC61E2::rsKoApxDTLYPn6H3NNT6iP2mv"
        );
        assert_eq!(
            auth.csrf_token().unwrap().as_str(),
            "4EEC61E2:lwk7od06fa1+DcPUwBTXCcndyAY/3mKxQp5vR8sNjWuBtL9fZg=="
        );
    }

    #[tokio::test]
    async fn test_login_service_unauthorized() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let connection = create_test_connection(&mock_server.uri()).await;
        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::Authentication(_))));
    }

    #[tokio::test]
    async fn test_login_service_bad_request() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&mock_server)
            .await;

        let connection = create_test_connection(&mock_server.uri()).await;
        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::Validation { .. })));
    }

    #[tokio::test]
    async fn test_login_service_not_found() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let connection = create_test_connection(&mock_server.uri()).await;
        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::Connection(_))));
    }

    #[tokio::test]
    async fn test_login_service_service_unavailable() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let connection = create_test_connection(&mock_server.uri()).await;
        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::Connection(_))));
    }

    #[tokio::test]
    async fn test_login_service_invalid_ticket_format() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "ticket": "invalid",
                    "CSRFPreventionToken": "4EEC61E2:valid"
                }
            })))
            .mount(&mock_server)
            .await;

        let connection = create_test_connection(&mock_server.uri()).await;
        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::Validation { .. })));
    }

    #[tokio::test]
    async fn test_login_service_invalid_csrf_format() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "ticket": "PVE:user@pam:4EEC61E2::sig",
                    "CSRFPreventionToken": "invalid"
                }
            })))
            .mount(&mock_server)
            .await;

        let connection = create_test_connection(&mock_server.uri()).await;
        let service = LoginService::new();
        let result = service.execute(&connection).await;
        assert!(matches!(result, Err(ProxmoxError::Validation { .. })));
    }
}
