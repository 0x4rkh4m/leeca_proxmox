//! Node management operations using the Proxmox client.
//!
//! This program authenticates against a Proxmox server,
//! lists all cluster nodes, retrieves detailed status
//! information for the first node, and fetches its DNS configuration.

use leeca_proxmox::{ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let host = "192.168.1.182";
    let port: u16 = 8006;
    let username = "leeca";
    let password = "password";
    let realm = "pam";

    // Build the client with node management configuration.
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

    // 1. List all nodes in the cluster.
    println!("\nListing all nodes:");
    let nodes = client.nodes().await?;

    for node in &nodes {
        println!(
            "  • {}: {} (CPU: {:.1}%, Memory: {:.1}/{:.1} GB, Uptime: {}s)",
            node.node,
            node.status,
            node.cpu.unwrap_or(0.0) * 100.0,
            node.mem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0,
            node.maxmem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0,
            node.uptime.unwrap_or(0)
        );
    }

    if nodes.is_empty() {
        println!("No nodes found.");
        return Ok(());
    }

    // 2. Retrieve detailed status for the first node.
    let first_node = &nodes[0].node;
    println!("\nDetailed status for node '{}':", first_node);

    let status = client.node_status(first_node).await?;

    println!(
        "  CPU: {:.2}%, IO Delay: {:.2}%",
        status.cpu * 100.0,
        status.wait.unwrap_or(0.0) * 100.0
    );

    println!(
        "  Memory: {:.1}/{:.1} GB used",
        status.memory.used as f64 / 1024.0 / 1024.0 / 1024.0,
        status.memory.total as f64 / 1024.0 / 1024.0 / 1024.0
    );

    if let Some(swap) = &status.swap {
        println!(
            "  Swap: {:.1}/{:.1} GB used",
            swap.used as f64 / 1024.0 / 1024.0 / 1024.0,
            swap.total as f64 / 1024.0 / 1024.0 / 1024.0
        );
    }

    println!("  Uptime: {} seconds", status.uptime);

    if let Some(loadavg) = status.loadavg {
        println!(
            "  Load average: {:.2}, {:.2}, {:.2}",
            loadavg[0], loadavg[1], loadavg[2]
        );
    }

    if let Some(kversion) = status.kversion {
        println!("  Kernel: {}", kversion);
    }

    // 3. Retrieve DNS configuration for the first node.
    println!("\nDNS configuration for node '{}':", first_node);

    let dns = client.node_dns(first_node).await?;

    println!("  Search domain: {}", dns.domain);
    println!("  DNS servers: {:?}", dns.servers);

    if let Some(options) = dns.options {
        println!("  Options: {:?}", options);
    }

    Ok(())
}
