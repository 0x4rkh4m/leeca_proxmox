use crate::{
    ProxmoxClient, ProxmoxConnection, ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm,
    ProxmoxUrl, ProxmoxUsername, ValidationConfig, core::domain::model::vm::*,
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

    use crate::core::domain::value_object::{ProxmoxCSRFToken, ProxmoxTicket};
    let ticket = ProxmoxTicket::new_unchecked("PVE:testuser@pam:4EEC61E2::sig".to_string());
    let csrf = ProxmoxCSRFToken::new_unchecked("4EEC61E2:token".to_string());
    let auth = crate::ProxmoxAuth::new(ticket, Some(csrf));
    client.set_auth(auth).await;
    client
}

#[tokio::test]
async fn test_vms_list_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/qemu"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "vmid": 100,
                    "name": "ubuntu-vm",
                    "status": "running",
                    "cpu": 0.23,
                    "maxcpu": 4,
                    "mem": 4294967296_i64,
                    "maxmem": 8589934592_i64,
                    "disk": 21474836480_i64,
                    "maxdisk": 42949672960_i64,
                    "uptime": 123456,
                    "node": "pve1",
                    "id": "qemu/100",
                    "tags": "ubuntu,production"
                },
                {
                    "vmid": 101,
                    "name": "windows-vm",
                    "status": "stopped",
                    "maxcpu": 8,
                    "maxmem": 17179869184_i64,
                    "maxdisk": 107374182400_i64,
                    "node": "pve1",
                    "id": "qemu/101"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let vms = proxmox_client.vms("pve1").await.unwrap();
    assert_eq!(vms.len(), 2);

    let vm1 = &vms[0];
    assert_eq!(vm1.vmid, 100);
    assert_eq!(vm1.name, "ubuntu-vm");
    assert_eq!(vm1.status, "running");
    assert_eq!(vm1.cpu, Some(0.23));
    assert_eq!(vm1.maxcpu, Some(4));
    assert_eq!(vm1.mem, Some(4294967296));
    assert_eq!(vm1.maxmem, Some(8589934592));
    assert_eq!(vm1.disk, Some(21474836480));
    assert_eq!(vm1.maxdisk, Some(42949672960));
    assert_eq!(vm1.uptime, Some(123456));
    assert_eq!(vm1.node, "pve1");
    assert_eq!(vm1.id, "qemu/100");
    assert_eq!(vm1.tags.as_deref(), Some("ubuntu,production"));

    let vm2 = &vms[1];
    assert_eq!(vm2.vmid, 101);
    assert_eq!(vm2.name, "windows-vm");
    assert_eq!(vm2.status, "stopped");
    assert_eq!(vm2.cpu, None);
    assert_eq!(vm2.maxcpu, Some(8));
    assert_eq!(vm2.mem, None);
    assert_eq!(vm2.maxmem, Some(17179869184));
    assert_eq!(vm2.disk, None);
    assert_eq!(vm2.maxdisk, Some(107374182400));
    assert_eq!(vm2.uptime, None);
    assert_eq!(vm2.node, "pve1");
    assert_eq!(vm2.id, "qemu/101");
    assert_eq!(vm2.tags, None);
}

#[tokio::test]
async fn test_vms_list_empty() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/qemu"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": []
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let vms = proxmox_client.vms("pve1").await.unwrap();
    assert!(vms.is_empty());
}

#[tokio::test]
async fn test_vm_status_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/current"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "status": "running",
                "name": "ubuntu-vm",
                "cpu": 0.15,
                "mem": 4294967296_i64,
                "uptime": 123456,
                "qmpstatus": "running",
                "balloon": {
                    "current": 4294967296_i64,
                    "maximum": 8589934592_i64,
                    "minimum": 1073741824_i64
                },
                "sockets": 1,
                "cores": 4,
                "cputype": "kvm64",
                "balloon_min": 1073741824_i64,
                "maxmem": 8589934592_i64,
                "freemem": 4294967296_i64,
                "totalmem": 8589934592_i64,
                "digest": "abc123"
            }
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let status = proxmox_client.vm_status("pve1", 100).await.unwrap();
    assert_eq!(status.status, "running");
    assert_eq!(status.name, "ubuntu-vm");
    assert_eq!(status.cpu, Some(0.15));
    assert_eq!(status.mem, Some(4294967296));
    assert_eq!(status.uptime, Some(123456));
    assert_eq!(status.qmpstatus.as_deref(), Some("running"));
    assert!(status.balloon.is_some());
    assert_eq!(status.sockets, Some(1));
    assert_eq!(status.cores, Some(4));
    assert_eq!(status.cpu_type.as_deref(), Some("kvm64"));
    assert_eq!(status.balloon_min, Some(1073741824));
    assert_eq!(status.maxmem, Some(8589934592));
    assert_eq!(status.freemem, Some(4294967296));
    assert_eq!(status.totalmem, Some(8589934592));
    assert_eq!(status.digest.as_deref(), Some("abc123"));
}

#[tokio::test]
async fn test_vm_status_minimal() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/current"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "status": "stopped",
                "name": "test-vm"
            }
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let status = proxmox_client.vm_status("pve1", 100).await.unwrap();
    assert_eq!(status.status, "stopped");
    assert_eq!(status.name, "test-vm");
    assert_eq!(status.cpu, None);
    assert_eq!(status.mem, None);
    assert_eq!(status.uptime, None);
    assert_eq!(status.qmpstatus, None);
}

#[tokio::test]
async fn test_start_vm_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/start"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:start"
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.start_vm("pve1", 100).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:start");
}

#[tokio::test]
async fn test_stop_vm_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/stop"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:stop"
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.stop_vm("pve1", 100).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:stop");
}

#[tokio::test]
async fn test_shutdown_vm_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/shutdown"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:shutdown"
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.shutdown_vm("pve1", 100).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:shutdown");
}

#[tokio::test]
async fn test_reboot_vm_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/reboot"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:reboot"
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.reboot_vm("pve1", 100).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:reboot");
}

#[tokio::test]
async fn test_reset_vm_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/reset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:reset"
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.reset_vm("pve1", 100).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:reset");
}

#[tokio::test]
async fn test_delete_vm_with_purge() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("DELETE"))
        .and(path("/api2/json/nodes/pve1/qemu/100"))
        .and(|req: &wiremock::Request| req.url.query() == Some("purge=1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:delete"
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.delete_vm("pve1", 100, true).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:delete");
}

#[tokio::test]
async fn test_delete_vm_without_purge() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("DELETE"))
        .and(path("/api2/json/nodes/pve1/qemu/100"))
        .and(|req: &wiremock::Request| req.url.query() == Some("purge=0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:delete"
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.delete_vm("pve1", 100, false).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:delete");
}

#[tokio::test]
async fn test_create_vm_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("POST"))
        .and(path("/api2/json/nodes/pve1/qemu"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:create"
        })))
        .mount(&mock_server)
        .await;

    let params = CreateVmParams {
        vmid: 100,
        name: "test-vm".to_string(),
        memory: Some(2048),
        sockets: Some(1),
        cores: Some(2),
        threads: None,
        cpu: Some("host".to_string()),
        ostype: Some("l26".to_string()),
        kvm: Some(1),
        numa: None,
        net: Some("virtio,bridge=vmbr0".to_string()),
        scsihw: Some("virtio-scsi-pci".to_string()),
        boot: Some("order=scsi0;net0".to_string()),
        start: Some(1),
        tags: Some("test".to_string()),
        description: Some("Created by leeca".to_string()),
        protection: None,
        tablet: Some(1),
        vga: Some("virtio".to_string()),
        bios: None,
        efidisk: None,
        tpmstate: None,
        agent: Some(1),
    };

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.create_vm("pve1", &params).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:create");
}

#[tokio::test]
async fn test_vm_config_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/api2/json/nodes/pve1/qemu/100/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "vmid": 100,
                "name": "ubuntu-vm",
                "description": "Production VM",
                "memory": 4096,
                "balloon": 1024,
                "sockets": 1,
                "cores": 4,
                "threads": 1,
                "cpu": "host",
                "net": "virtio,bridge=vmbr0",
                "scsi": "virtio,size=32G",
                "boot": "order=scsi0;net0",
                "ostype": "l26",
                "agent": 1,
                "kvm": 1,
                "numa": 0,
                "tags": "ubuntu,production",
                "onboot": 1,
                "protection": 0,
                "tablet": 1,
                "vga": "virtio",
                "scsihw": "virtio-scsi-pci",
                "bios": "seabios",
                "digest": "abc123def456"
            }
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let config = proxmox_client.vm_config("pve1", 100).await.unwrap();
    assert_eq!(config.vmid, 100);
    assert_eq!(config.name, "ubuntu-vm");
    assert_eq!(config.description.as_deref(), Some("Production VM"));
    assert_eq!(config.memory, Some(4096));
    assert_eq!(config.balloon, Some(1024));
    assert_eq!(config.sockets, Some(1));
    assert_eq!(config.cores, Some(4));
    assert_eq!(config.threads, Some(1));
    assert_eq!(config.cpu.as_deref(), Some("host"));
    assert!(config.net.is_some());
    assert!(config.scsi.is_some());
    assert_eq!(config.boot.as_deref(), Some("order=scsi0;net0"));
    assert_eq!(config.ostype.as_deref(), Some("l26"));
    assert_eq!(config.agent, Some(1));
    assert_eq!(config.kvm, Some(1));
    assert_eq!(config.numa, Some(0));
    assert_eq!(config.tags.as_deref(), Some("ubuntu,production"));
    assert_eq!(config.onboot, Some(1));
    assert_eq!(config.protection, Some(0));
    assert_eq!(config.tablet, Some(1));
    assert_eq!(config.vga.as_deref(), Some("virtio"));
    assert_eq!(config.scsihw.as_deref(), Some("virtio-scsi-pci"));
    assert_eq!(config.bios.as_deref(), Some("seabios"));
    assert_eq!(config.digest.as_deref(), Some("abc123def456"));
}

#[tokio::test]
async fn test_update_vm_config_success() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    Mock::given(method("PUT"))
        .and(path("/api2/json/nodes/pve1/qemu/100/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:update"
        })))
        .mount(&mock_server)
        .await;

    let params = CreateVmParams {
        vmid: 100,
        name: "ubuntu-vm".to_string(),
        memory: Some(8192),
        sockets: Some(2),
        cores: Some(4),
        threads: None,
        cpu: None,
        ostype: None,
        kvm: None,
        numa: None,
        net: None,
        scsihw: None,
        boot: None,
        start: None,
        tags: Some("updated".to_string()),
        description: Some("Updated description".to_string()),
        protection: Some(1),
        tablet: None,
        vga: None,
        bios: None,
        efidisk: None,
        tpmstate: None,
        agent: None,
    };

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client
        .update_vm_config("pve1", 100, &params)
        .await
        .unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:update");
}

#[tokio::test]
async fn test_vm_actions_unauthorized_triggers_refresh() {
    let mock_server = MockServer::start().await;
    let client = create_authenticated_client(&mock_server).await;

    // First request returns 401
    Mock::given(method("POST"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/start"))
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
    Mock::given(method("POST"))
        .and(path("/api2/json/nodes/pve1/qemu/100/status/start"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "UPID:pve1:00000001:00000001:00000001:start"
        })))
        .mount(&mock_server)
        .await;

    let proxmox_client = ProxmoxClient {
        api_client: client,
        config: ValidationConfig::default(),
    };

    let task_id = proxmox_client.start_vm("pve1", 100).await.unwrap();
    assert_eq!(task_id, "UPID:pve1:00000001:00000001:00000001:start");
}
