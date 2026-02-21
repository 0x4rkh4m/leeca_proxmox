//! Internal HTTP client that handles authentication and automatic ticket refresh.

use crate::{
    ProxmoxAuth, ProxmoxConnection, ProxmoxError, ProxmoxResult, ValidationConfig,
    auth::application::service::login_service::LoginService,
};
use governor::{DefaultDirectRateLimiter, Quota};
use reqwest::{Client, StatusCode};
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Internal HTTP client that manages authentication and provides methods to call the Proxmox API.
///
/// This client automatically adds the necessary authentication headers (`PVEAuthCookie` and
/// `CSRFPreventionToken`) to each request. If a request receives a `401 Unauthorized` response,
/// it attempts to refresh the ticket once using the stored credentials and retries the request.
#[derive(Debug)]
pub struct ApiClient {
    http_client: Client,
    connection: Arc<ProxmoxConnection>,
    auth: Arc<RwLock<Option<ProxmoxAuth>>>,
    config: Arc<ValidationConfig>,
    rate_limiter: Option<Arc<DefaultDirectRateLimiter>>,
}

impl ApiClient {
    /// Creates a new `ApiClient`. The client starts unauthenticated.
    ///
    /// # Errors
    /// Returns `ProxmoxError::Connection` if the HTTP client cannot be built.
    pub fn new(connection: ProxmoxConnection, config: ValidationConfig) -> ProxmoxResult<Self> {
        let http_client = Client::builder()
            .danger_accept_invalid_certs(connection.accept_invalid_certs())
            .build()
            .map_err(|e| ProxmoxError::Connection(e.to_string()))?;

        let rate_limiter = config.rate_limit.map(|rl| {
            let quota = Quota::per_second(NonZeroU32::new(rl.requests_per_second).unwrap())
                .allow_burst(NonZeroU32::new(rl.burst_size).unwrap());
            Arc::new(DefaultDirectRateLimiter::direct(quota))
        });

        Ok(Self {
            http_client,
            connection: Arc::new(connection),
            auth: Arc::new(RwLock::new(None)),
            config: Arc::new(config),
            rate_limiter,
        })
    }

    /// Returns a reference to the underlying connection details.
    pub fn connection(&self) -> &ProxmoxConnection {
        &self.connection
    }

    /// Sets the authentication state (used after a successful login or session restore).
    pub async fn set_auth(&self, auth: ProxmoxAuth) {
        let mut lock = self.auth.write().await;
        *lock = Some(auth);
    }

    /// Returns the current authentication state, if any.
    pub async fn auth(&self) -> Option<ProxmoxAuth> {
        self.auth.read().await.clone()
    }

    /// Returns `true` if there is a valid (non‑expired) ticket.
    pub async fn is_authenticated(&self) -> bool {
        let lock = self.auth.read().await;
        lock.as_ref()
            .map(|a| !a.ticket().is_expired(self.config.ticket_lifetime))
            .unwrap_or(false)
    }

    /// Performs an authenticated GET request.
    ///
    /// # Type Parameters
    /// - `T`: The expected response type (must implement `DeserializeOwned`).
    ///
    /// # Errors
    /// Returns `ProxmoxError` if the request fails, authentication cannot be refreshed,
    /// or the response cannot be parsed.
    #[allow(dead_code)] // Will be used in future resource operations
    pub async fn get<T>(&self, path: &str) -> ProxmoxResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.execute_request(reqwest::Method::GET, path, None::<&()>)
            .await
    }

    /// Performs an authenticated POST request with a JSON body.
    ///
    /// # Type Parameters
    /// - `B`: The body type (must implement `Serialize`).
    /// - `T`: The expected response type (must implement `DeserializeOwned`).
    ///
    /// # Errors
    /// Returns `ProxmoxError` if the request fails, authentication cannot be refreshed,
    /// or the response cannot be parsed.
    #[allow(dead_code)] // Will be used in future resource operations
    pub async fn post<B, T>(&self, path: &str, body: &B) -> ProxmoxResult<T>
    where
        B: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        self.execute_request(reqwest::Method::POST, path, Some(body))
            .await
    }

    /// Performs an authenticated PUT request with a JSON body.
    ///
    /// # Type Parameters
    /// - `B`: The body type (must implement `Serialize`).
    /// - `T`: The expected response type (must implement `DeserializeOwned`).
    ///
    /// # Errors
    /// Returns `ProxmoxError` if the request fails, authentication cannot be refreshed,
    /// or the response cannot be parsed.
    #[allow(dead_code)] // Will be used in future resource operations
    pub async fn put<B, T>(&self, path: &str, body: &B) -> ProxmoxResult<T>
    where
        B: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        self.execute_request(reqwest::Method::PUT, path, Some(body))
            .await
    }

    /// Performs an authenticated DELETE request.
    ///
    /// # Type Parameters
    /// - `T`: The expected response type (must implement `DeserializeOwned`).
    ///
    /// # Errors
    /// Returns `ProxmoxError` if the request fails, authentication cannot be refreshed,
    /// or the response cannot be parsed.
    #[allow(dead_code)] // Will be used in future resource operations
    pub async fn delete<T>(&self, path: &str) -> ProxmoxResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.execute_request(reqwest::Method::DELETE, path, None::<&()>)
            .await
    }

    /// Core request execution method. It ensures authentication, sends the request,
    /// handles 401 by refreshing once, and parses the response.
    async fn execute_request<B, T>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> ProxmoxResult<T>
    where
        B: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        // Ensure we have a valid ticket (refresh if needed)
        self.ensure_authenticated().await?;

        // Apply rate limiting if enabled
        if let Some(limiter) = &self.rate_limiter {
            // `until_ready()` returns a future that completes when capacity is available.
            limiter.until_ready().await;
        }

        // Build the full URL
        let base = self.connection.url().as_str().trim_end_matches('/');
        let url = format!("{}/api2/json/{}", base, path.trim_start_matches('/'));

        // Start building the request (clone method to avoid move)
        let mut req_builder = self.http_client.request(method.clone(), &url);

        // Add authentication headers
        {
            let auth_guard = self.auth.read().await;
            if let Some(auth) = auth_guard.as_ref() {
                req_builder = req_builder
                    .header("Cookie", auth.ticket().as_cookie_header())
                    .header("CSRFPreventionToken", auth.csrf_token().unwrap().as_str());
            }
        }

        // Add body if present
        if let Some(body) = body {
            req_builder = req_builder.json(body);
        }

        // Send the request
        let response = req_builder
            .send()
            .await
            .map_err(|e| ProxmoxError::Connection(format!("HTTP request failed: {}", e)))?;

        // Handle 401 Unauthorized: refresh once and retry
        if response.status() == StatusCode::UNAUTHORIZED {
            self.refresh_auth().await?;
            // Retry exactly once (no further recursion)
            return self.retry_request(method, path, body).await;
        }

        // Handle other HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(ProxmoxError::Connection(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Parse successful response
        response
            .json::<T>()
            .await
            .map_err(|e| ProxmoxError::Connection(format!("Failed to parse response: {}", e)))
    }

    /// Retry a request after a successful token refresh. This method avoids recursion.
    async fn retry_request<B, T>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> ProxmoxResult<T>
    where
        B: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let base = self.connection.url().as_str().trim_end_matches('/');
        let url = format!("{}/api2/json/{}", base, path.trim_start_matches('/'));

        let mut req_builder = self.http_client.request(method, &url);

        // Authentication headers must be present after refresh
        {
            let auth_guard = self.auth.read().await;
            if let Some(auth) = auth_guard.as_ref() {
                req_builder = req_builder
                    .header("Cookie", auth.ticket().as_cookie_header())
                    .header("CSRFPreventionToken", auth.csrf_token().unwrap().as_str());
            }
        }

        if let Some(body) = body {
            req_builder = req_builder.json(body);
        }

        let response = req_builder.send().await.map_err(|e| {
            ProxmoxError::Connection(format!("HTTP request failed on retry: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(ProxmoxError::Connection(format!(
                "API error after refresh ({}): {}",
                status, error_text
            )));
        }

        response.json::<T>().await.map_err(|e| {
            ProxmoxError::Connection(format!("Failed to parse response after refresh: {}", e))
        })
    }

    /// Ensures that we have a valid (non‑expired) ticket. If not, attempts to refresh.
    async fn ensure_authenticated(&self) -> ProxmoxResult<()> {
        let need_refresh = {
            let auth_guard = self.auth.read().await;
            match auth_guard.as_ref() {
                Some(auth) => auth.ticket().is_expired(self.config.ticket_lifetime),
                None => true,
            }
        };

        if need_refresh {
            self.refresh_auth().await?;
        }
        Ok(())
    }

    /// Performs a fresh login using the stored credentials to obtain a new ticket.
    async fn refresh_auth(&self) -> ProxmoxResult<()> {
        let service = LoginService::new();
        let auth = service.execute(&self.connection).await?;
        let mut lock = self.auth.write().await;
        *lock = Some(auth);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm, ProxmoxUrl, ProxmoxUsername,
        RateLimitConfig,
        core::domain::value_object::{ProxmoxCSRFToken, ProxmoxTicket},
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    fn create_test_connection(server_url: &str) -> ProxmoxConnection {
        let host = ProxmoxHost::new_unchecked(server_url.trim_start_matches("http://").to_string());
        let port = ProxmoxPort::new_unchecked(8006);
        let username = ProxmoxUsername::new_unchecked("testuser".to_string());
        let password = ProxmoxPassword::new_unchecked("testpass".to_string());
        let realm = ProxmoxRealm::new_unchecked("pam".to_string());
        let url = ProxmoxUrl::new_unchecked(server_url.to_string() + "/");
        ProxmoxConnection::new(host, port, username, password, realm, false, true, url)
    }

    fn create_test_auth() -> ProxmoxAuth {
        let ticket = ProxmoxTicket::new_unchecked("PVE:testuser@pam:4EEC61E2::sig".to_string());
        let csrf = ProxmoxCSRFToken::new_unchecked("4EEC61E2:token".to_string());
        ProxmoxAuth::new(ticket, Some(csrf))
    }

    #[tokio::test]
    async fn test_get_success() {
        let mock_server = MockServer::start().await;
        let connection = create_test_connection(&mock_server.uri());
        let config = ValidationConfig::default();
        let client = ApiClient::new(connection, config).unwrap();

        // Pre‑authenticate
        client.set_auth(create_test_auth()).await;

        Mock::given(method("GET"))
            .and(path("/api2/json/test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": "ok"})),
            )
            .mount(&mock_server)
            .await;

        let result: serde_json::Value = client.get("test").await.unwrap();
        assert_eq!(result["data"], "ok");
    }

    #[tokio::test]
    async fn test_unauthorized_triggers_refresh() {
        let mock_server = MockServer::start().await;
        let connection = create_test_connection(&mock_server.uri());
        let config = ValidationConfig::default();
        let client = ApiClient::new(connection, config).unwrap();

        // First GET returns 401
        Mock::given(method("GET"))
            .and(path("/api2/json/test"))
            .respond_with(ResponseTemplate::new(401))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Login endpoint returns a valid ticket and CSRF token
        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "ticket": "PVE:testuser@pam:4EEC61E2::new_sig",
                    "CSRFPreventionToken": "4EEC61E2:abc123"   // valid token (alphanumeric)
                }
            })))
            .mount(&mock_server)
            .await;

        // Second GET (retry) returns 200
        Mock::given(method("GET"))
            .and(path("/api2/json/test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": "ok"})),
            )
            .mount(&mock_server)
            .await;

        let result: serde_json::Value = client.get("test").await.unwrap();
        assert_eq!(result["data"], "ok");

        // Verify that new auth was stored.
        let auth = client.auth().await.unwrap();
        assert_eq!(auth.ticket().as_str(), "PVE:testuser@pam:4EEC61E2::new_sig");
        assert_eq!(auth.csrf_token().unwrap().as_str(), "4EEC61E2:abc123");
    }

    #[tokio::test]
    async fn test_refresh_failure_returns_error() {
        let mock_server = MockServer::start().await;
        let connection = create_test_connection(&mock_server.uri());
        let config = ValidationConfig::default();
        let client = ApiClient::new(connection, config).unwrap();

        // First request returns 401, then login endpoint returns 401 as well.
        Mock::given(method("GET"))
            .and(path("/api2/json/test"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api2/json/access/ticket"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let result: ProxmoxResult<serde_json::Value> = client.get("test").await;
        assert!(matches!(result, Err(ProxmoxError::Authentication(_))));
    }

    #[tokio::test]
    async fn test_rate_limiting_delays_requests() {
        use std::time::{Duration, Instant};

        let mock_server = MockServer::start().await;
        let connection = create_test_connection(&mock_server.uri());
        let config = ValidationConfig {
            rate_limit: Some(RateLimitConfig {
                requests_per_second: 2,
                burst_size: 2,
            }),
            ..Default::default()
        };
        let client = ApiClient::new(connection, config).unwrap();

        // Pre-authenticate
        client.set_auth(create_test_auth()).await;

        // Mock endpoint returns 200 quickly
        Mock::given(method("GET"))
            .and(path("/api2/json/test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": "ok"})),
            )
            .expect(4) // we will send 4 requests
            .mount(&mock_server)
            .await;

        // Send first two requests immediately – they should pass without delay (burst)
        let start = Instant::now();
        let req1 = client.get::<serde_json::Value>("test");
        let req2 = client.get::<serde_json::Value>("test");
        let (res1, res2) = tokio::join!(req1, req2);
        res1.unwrap();
        res2.unwrap();
        let elapsed = start.elapsed();
        // Relajamos el umbral para CI
        assert!(elapsed < Duration::from_millis(500)); // should be nearly instant

        // Send third and fourth requests – they should be delayed to respect the 2/sec rate
        let start = Instant::now();
        let req3 = client.get::<serde_json::Value>("test");
        let req4 = client.get::<serde_json::Value>("test");
        let (res3, res4) = tokio::join!(req3, req4);
        res3.unwrap();
        res4.unwrap();
        let elapsed = start.elapsed();
        // We verify that there is a total delay of approximately 1 second.
        assert!(elapsed >= Duration::from_millis(900));
    }

    #[tokio::test]
    async fn test_rate_limiting_disabled() {
        use tokio::time::{self, Duration};

        let mock_server = MockServer::start().await;
        let connection = create_test_connection(&mock_server.uri());
        let config = ValidationConfig {
            rate_limit: None, // disabled
            ..Default::default()
        };
        let client = ApiClient::new(connection, config).unwrap();
        client.set_auth(create_test_auth()).await;

        Mock::given(method("GET"))
            .and(path("/api2/json/test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": "ok"})),
            )
            .expect(10)
            .mount(&mock_server)
            .await;

        let start = time::Instant::now();
        for _ in 0..10 {
            client.get::<serde_json::Value>("test").await.unwrap();
        }
        let elapsed = start.elapsed();
        // Should be very fast (no delays)
        assert!(elapsed < Duration::from_millis(500));
    }
}
