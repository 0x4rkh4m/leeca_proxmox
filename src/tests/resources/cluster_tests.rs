use crate::{
    ProxmoxClient, ProxmoxConnection, ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm,
    ProxmoxUrl, ProxmoxUsername, ValidationConfig,
    core::domain::model::cluster_resource::ClusterResource,
    core::infrastructure::api_client::ApiClient,
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

    // Set a dummy auth (we don't need real login for these tests)
    use crate::core::domain::value_object::{ProxmoxCSRFToken, ProxmoxTicket};
    let ticket = ProxmoxTicket::new_unchecked("PVE:testuser@pam:4EEC61E2::sig".to_string());
    let csrf = ProxmoxCSRFToken::new_unchecked("4EEC61E2:token".to_string());
    let auth = crate::ProxmoxAuth::new(ticket, Some(csrf));
    client.set_auth(auth).await;
    client
}

#[tokio::test]
async fn test_cluster_resources_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    // Mock response from Proxmox
    Mock::given(method("GET"))
        .and(path("/api2/json/cluster/resources"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "type": "qemu",
                    "vmid": 100,
                    "node": "pve1",
                    "id": "qemu/100",
                    "name": "ubuntu-vm",
                    "status": "running",
                    "maxcpu": 4,
                    "maxmem": 8589934592_i64,
                    "disk": 21474836480_i64,
                    "uptime": 123456
                },
                {
                    "type": "lxc",
                    "vmid": 200,
                    "node": "pve2",
                    "id": "lxc/200",
                    "name": "alpine-ct",
                    "status": "stopped",
                    "maxcpu": 2,
                    "maxmem": 1073741824_i64,
                    "disk": 5368709120_i64,
                    "swap": 536870912_i64,
                    "uptime": 0
                },
                {
                    "type": "storage",
                    "storage": "local",
                    "node": "pve1",
                    "id": "storage/local",
                    "status": "available",
                    "plugintype": "dir",
                    "total": 1099511627776_i64,
                    "used": 536870912000_i64,
                    "avail": 562640715776_i64
                },
                {
                    "type": "node",
                    "node": "pve1",
                    "id": "node/pve1",
                    "status": "online",
                    "cpu": 0.15,
                    "mem": 0.42,
                    "loadavg": [0.5, 0.3, 0.1],
                    "kversion": "Linux 5.15.30-1-pve",
                    "uptime": 999999
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // Use the client through ProxmoxClient wrapper
    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let resources = proxmox_client.cluster_resources().await.unwrap();
    assert_eq!(resources.len(), 4);

    // Check first resource (QEMU)
    match &resources[0] {
        ClusterResource::Qemu(vm) => {
            assert_eq!(vm.common.node, "pve1");
            assert_eq!(vm.vmid, 100);
            assert_eq!(vm.common.name.as_deref(), Some("ubuntu-vm"));
            assert_eq!(vm.common.status, "running");
            assert_eq!(vm.maxcpu, Some(4));
            assert_eq!(vm.maxmem, Some(8589934592));
            assert_eq!(vm.disk, Some(21474836480));
            assert_eq!(vm.common.uptime, Some(123456));
        }
        _ => panic!("Expected Qemu resource"),
    }

    // Check second resource (LXC)
    match &resources[1] {
        ClusterResource::Lxc(ct) => {
            assert_eq!(ct.common.node, "pve2");
            assert_eq!(ct.vmid, 200);
            assert_eq!(ct.common.name.as_deref(), Some("alpine-ct"));
            assert_eq!(ct.common.status, "stopped");
            assert_eq!(ct.maxcpu, Some(2));
            assert_eq!(ct.maxmem, Some(1073741824));
            assert_eq!(ct.disk, Some(5368709120));
            assert_eq!(ct.swap, Some(536870912));
            assert_eq!(ct.common.uptime, Some(0));
        }
        _ => panic!("Expected Lxc resource"),
    }

    // Check third resource (Storage)
    match &resources[2] {
        ClusterResource::Storage(st) => {
            assert_eq!(st.common.node, "pve1");
            assert_eq!(st.storage, "local");
            assert_eq!(st.storage_type, "dir");
            assert_eq!(st.common.status, "available");
            assert_eq!(st.total, Some(1099511627776));
            assert_eq!(st.used, Some(536870912000));
            assert_eq!(st.avail, Some(562640715776));
        }
        _ => panic!("Expected Storage resource"),
    }

    // Check fourth resource (Node)
    match &resources[3] {
        ClusterResource::Node(node) => {
            assert_eq!(node.common.node, "pve1");
            assert_eq!(node.common.status, "online");
            assert_eq!(node.cpu, Some(0.15));
            assert_eq!(node.mem, Some(0.42));
            assert_eq!(node.loadavg, Some([0.5, 0.3, 0.1]));
            assert_eq!(node.kversion.as_deref(), Some("Linux 5.15.30-1-pve"));
            assert_eq!(node.common.uptime, Some(999999));
        }
        _ => panic!("Expected Node resource"),
    }
}

#[tokio::test]
async fn test_cluster_resources_empty() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/cluster/resources"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": []
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let resources = proxmox_client.cluster_resources().await.unwrap();
    assert!(resources.is_empty());
}

#[tokio::test]
async fn test_cluster_resources_unauthorized_triggers_refresh() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    // First request returns 401
    Mock::given(method("GET"))
        .and(path("/api2/json/cluster/resources"))
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
        .and(path("/api2/json/cluster/resources"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "type": "qemu",
                    "vmid": 100,
                    "node": "pve1",
                    "id": "qemu/100",
                    "name": "retry-vm",
                    "status": "running"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let resources = proxmox_client.cluster_resources().await.unwrap();
    assert_eq!(resources.len(), 1);
    match &resources[0] {
        ClusterResource::Qemu(vm) => {
            assert_eq!(vm.common.name.as_deref(), Some("retry-vm"));
        }
        _ => panic!("Expected Qemu resource"),
    }
}
