//! Virtual machine management operations using the Proxmox client.
//!
//! This program authenticates against a Proxmox server,
//! lists virtual machines on a node, retrieves detailed VM
//! information, and demonstrates lifecycle and creation operations.

use leeca_proxmox::{CreateVmParams, ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let host = "192.168.1.182";
    let port: u16 = 8006;
    let username = "leeca";
    let password = "password";
    let realm = "pam";

    // Build the client with VM management configuration.
    let mut client = ProxmoxClient::builder()
        .host(host)
        .port(port)
        .credentials(username, password, realm)
        .secure(false) // HTTP for local development
        .accept_invalid_certs(true) // Testing & self-signed certs
        .build()
        .await?;

    // Authenticate against the Proxmox API.
    client.login().await?;
    println!("Authenticated successfully");

    // Target node.
    let node = "pve1";

    // 1. List all VMs on the node.
    println!("\nListing VMs on node '{}':", node);
    let vms = client.vms(node).await?;

    for vm in &vms {
        println!(
            "  • VM {} (ID: {}): {} - CPU: {:.1}%, Mem: {:.1}/{:.1} GB, Uptime: {}s",
            vm.name,
            vm.vmid,
            vm.status,
            vm.cpu.unwrap_or(0.0) * 100.0,
            vm.mem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0,
            vm.maxmem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0,
            vm.uptime.unwrap_or(0)
        );
    }

    if vms.is_empty() {
        println!("No VMs found on node '{}'.", node);
    }

    // 2. Retrieve detailed information for the first VM.
    if let Some(first_vm) = vms.first() {
        let vmid = first_vm.vmid;

        println!("\nDetailed status for VM {} (ID: {}):", first_vm.name, vmid);
        let status = client.vm_status(node, vmid).await?;

        println!("  Status: {}", status.status);

        if let Some(cpu) = status.cpu {
            println!("  CPU: {:.2}%", cpu * 100.0);
        }

        if let Some(mem) = status.mem {
            println!(
                "  Memory: {:.1} GB used",
                mem as f64 / 1024.0 / 1024.0 / 1024.0
            );
        }

        if let Some(uptime) = status.uptime {
            println!("  Uptime: {} seconds", uptime);
        }

        if let Some(balloon) = status.balloon {
            println!(
                "  Balloon: current={:.1} MB",
                balloon.current as f64 / 1024.0 / 1024.0
            );
        }

        // Retrieve VM configuration.
        println!("\nConfiguration for VM {}:", first_vm.name);
        let config = client.vm_config(node, vmid).await?;

        println!("  Memory: {} MB", config.memory.unwrap_or(0));
        println!(
            "  CPU: {} sockets x {} cores",
            config.sockets.unwrap_or(1),
            config.cores.unwrap_or(1)
        );
        println!(
            "  OS Type: {}",
            config.ostype.as_deref().unwrap_or("unknown")
        );
        println!("  Tags: {}", config.tags.as_deref().unwrap_or("none"));

        // Lifecycle operations (disabled by default).
        /*
        println!("\nStopping VM...");
        let task = client.stop_vm(node, vmid).await?;
        println!("  Task ID: {}", task);

        println!("\nStarting VM...");
        let task = client.start_vm(node, vmid).await?;
        println!("  Task ID: {}", task);
        */
    }

    // 3. Prepare parameters for VM creation.
    println!("\nCreating a new VM (not executed):");

    let params = CreateVmParams {
        vmid: 9999,
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
        start: Some(0),
        tags: Some("example".to_string()),
        description: Some("Created via ProxmoxClient".to_string()),
        protection: None,
        tablet: Some(1),
        vga: Some("virtio".to_string()),
        bios: None,
        efidisk: None,
        tpmstate: None,
        agent: Some(1),
    };

    println!(
        "  Prepared VM creation request for ID {} with name '{}'",
        params.vmid, params.name
    );

    // Uncomment to execute:
    // let task = client.create_vm(node, &params).await?;
    // println!("  Task ID: {}", task);

    Ok(())
}
