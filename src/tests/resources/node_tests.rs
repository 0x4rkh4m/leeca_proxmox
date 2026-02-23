use crate::{
    ProxmoxClient, ProxmoxConnection, ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm,
    ProxmoxUrl, ProxmoxUsername, ValidationConfig, core::infrastructure::api_client::ApiClient,
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

async fn create_authenticated_client(mock_server: &MockServer) -> ApiClient {
    let connection = create_test_connection(&mock_server.uri());
    let config = ValidationConfig::default();
    let client = ApiClient::new(connection, config).unwrap();

    use crate::core::domain::value_object::{ProxmoxCSRFToken, ProxmoxTicket};
    let ticket = ProxmoxTicket::new_unchecked("PVE:testuser@pam:4EEC61E2::sig".to_string());
    let csrf = ProxmoxCSRFToken::new_unchecked("4EEC61E2:token".to_string());
    let auth = crate::ProxmoxAuth::new(ticket, Some(csrf));
    client.set_auth(auth).await;
    client
}

#[tokio::test]
async fn test_nodes_list_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "node": "pve1",
                    "status": "online",
                    "cpu": 0.15,
                    "maxcpu": 8,
                    "mem": 8589934592_i64,
                    "maxmem": 17179869184_i64,
                    "disk": 1099511627776_i64,
                    "maxdisk": 2199023255552_i64,
                    "uptime": 1234567,
                    "id": "node/pve1",
                    "ssl_fingerprint": "AA:BB:CC:DD:EE:FF"
                },
                {
                    "node": "pve2",
                    "status": "online",
                    "cpu": 0.08,
                    "maxcpu": 16,
                    "mem": 4294967296_i64,
                    "maxmem": 34359738368_i64,
                    "disk": 549755813888_i64,
                    "maxdisk": 4398046511104_i64,
                    "uptime": 987654,
                    "id": "node/pve2"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let nodes = proxmox_client.nodes().await.unwrap();
    assert_eq!(nodes.len(), 2);

    // Check first node
    let node1 = &nodes[0];
    assert_eq!(node1.node, "pve1");
    assert_eq!(node1.status, "online");
    assert_eq!(node1.cpu, Some(0.15));
    assert_eq!(node1.maxcpu, Some(8));
    assert_eq!(node1.mem, Some(8589934592));
    assert_eq!(node1.maxmem, Some(17179869184));
    assert_eq!(node1.disk, Some(1099511627776));
    assert_eq!(node1.maxdisk, Some(2199023255552));
    assert_eq!(node1.uptime, Some(1234567));
    assert_eq!(node1.id.as_deref(), Some("node/pve1"));
    assert_eq!(node1.ssl_fingerprint.as_deref(), Some("AA:BB:CC:DD:EE:FF"));

    // Check second node
    let node2 = &nodes[1];
    assert_eq!(node2.node, "pve2");
    assert_eq!(node2.status, "online");
    assert_eq!(node2.cpu, Some(0.08));
    assert_eq!(node2.maxcpu, Some(16));
    assert_eq!(node2.mem, Some(4294967296));
    assert_eq!(node2.maxmem, Some(34359738368));
    assert_eq!(node2.disk, Some(549755813888));
    assert_eq!(node2.maxdisk, Some(4398046511104));
    assert_eq!(node2.uptime, Some(987654));
    assert_eq!(node2.id.as_deref(), Some("node/pve2"));
    assert_eq!(node2.ssl_fingerprint, None);
}

#[tokio::test]
async fn test_nodes_list_empty() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": []
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let nodes = proxmox_client.nodes().await.unwrap();
    assert!(nodes.is_empty());
}

#[tokio::test]
async fn test_node_status_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "cpu": 0.22,
                "memory": {
                    "total": 17179869184_i64,
                    "used": 8589934592_i64,
                    "free": 8589934592_i64
                },
                "swap": {
                    "total": 4294967296_i64,
                    "used": 0,
                    "free": 4294967296_i64
                },
                "uptime": 1234567,
                "kversion": "Linux 5.15.30-1-pve",
                "loadavg": [1.2, 0.8, 0.5],
                "current-kernel": "5.15.30-1-pve",
                "description": "Main production node",
                "wait": 0.03,
                "cpuinfo": "Intel(R) Xeon(R) CPU E5-2680 v4 @ 2.40GHz",
                "pve-version": "7.3-1"
            }
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let status = proxmox_client.node_status("pve1").await.unwrap();
    assert_eq!(status.cpu, 0.22);
    assert_eq!(status.memory.total, 17179869184);
    assert_eq!(status.memory.used, 8589934592);
    assert_eq!(status.memory.free, 8589934592);
    assert_eq!(status.swap.as_ref().unwrap().total, 4294967296);
    assert_eq!(status.swap.as_ref().unwrap().used, 0);
    assert_eq!(status.swap.as_ref().unwrap().free, 4294967296);
    assert_eq!(status.uptime, 1234567);
    assert_eq!(status.kversion.as_deref(), Some("Linux 5.15.30-1-pve"));
    assert_eq!(status.loadavg, Some([1.2, 0.8, 0.5]));
    assert_eq!(status.wait, Some(0.03));
    assert_eq!(
        status.cpuinfo.as_deref(),
        Some("Intel(R) Xeon(R) CPU E5-2680 v4 @ 2.40GHz")
    );
    assert_eq!(status.pve_version.as_deref(), Some("7.3-1"));
}

#[tokio::test]
async fn test_node_status_without_optional_fields() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "cpu": 0.15,
                "memory": {
                    "total": 17179869184_i64,
                    "used": 8589934592_i64,
                    "free": 8589934592_i64
                },
                "uptime": 1234567,
                "kversion": "Linux 5.15.30-1-pve"
            }
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let status = proxmox_client.node_status("pve1").await.unwrap();
    assert_eq!(status.cpu, 0.15);
    assert_eq!(status.memory.total, 17179869184);
    assert_eq!(status.memory.used, 8589934592);
    assert_eq!(status.memory.free, 8589934592);
    assert_eq!(status.uptime, 1234567);
    assert_eq!(status.kversion.as_deref(), Some("Linux 5.15.30-1-pve"));
    assert_eq!(status.swap, None);
    assert_eq!(status.loadavg, None);
    assert_eq!(status.wait, None);
    assert_eq!(status.cpuinfo, None);
    assert_eq!(status.pve_version, None);
}

#[tokio::test]
async fn test_node_dns_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/dns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "domain": "example.com",
                "servers": ["8.8.8.8", "8.8.4.4"],
                "options": ["rotate", "timeout:2"]
            }
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let dns = proxmox_client.node_dns("pve1").await.unwrap();
    assert_eq!(dns.domain, "example.com");
    assert_eq!(
        dns.servers,
        vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()]
    );
    assert_eq!(
        dns.options,
        Some(vec!["rotate".to_string(), "timeout:2".to_string()])
    );
}

#[tokio::test]
async fn test_node_dns_without_options() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/dns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "domain": "example.com",
                "servers": ["8.8.8.8"]
            }
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let dns = proxmox_client.node_dns("pve1").await.unwrap();
    assert_eq!(dns.domain, "example.com");
    assert_eq!(dns.servers, vec!["8.8.8.8".to_string()]);
    assert_eq!(dns.options, None);
}

#[tokio::test]
async fn test_node_dns_unauthorized_triggers_refresh() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    // First request returns 401
    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/dns"))
        .respond_with(ResponseTemplate::new(401))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Login endpoint returns a new ticket
    Mock::given(method("POST"))
        .and(path("/api2/json/access/ticket"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "ticket": "PVE:testuser@pam:4EEC61E2::refreshed",
                "CSRFPreventionToken": "4EEC61E2:newtoken"
            }
        })))
        .mount(&mock_server)
        .await;

    // Second (retry) request succeeds
    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/dns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "domain": "example.com",
                "servers": ["8.8.8.8"]
            }
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let dns = proxmox_client.node_dns("pve1").await.unwrap();
    assert_eq!(dns.domain, "example.com");
    assert_eq!(dns.servers, vec!["8.8.8.8".to_string()]);
}
