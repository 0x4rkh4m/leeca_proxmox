//! Discover all resources available in the Proxmox cluster.
//!
//! This program authenticates against a Proxmox server and retrieves
//! every resource in the cluster, including virtual machines,
//! containers, storage backends, and nodes.

use leeca_proxmox::{ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let host = "192.168.1.182";
    let port: u16 = 8006;
    let username = "leeca";
    let password = "password";
    let realm = "pam";

    // Build the client with cluster connection settings.
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

    // Fetch all cluster resources.
    let resources = client.cluster_resources().await?;
    println!("Found {} resources in cluster\n", resources.len());

    // Counters for summary output.
    let mut vms = 0;
    let mut containers = 0;
    let mut storages = 0;
    let mut nodes = 0;

    // Categorize and display resources.
    for resource in resources {
        use leeca_proxmox::core::domain::model::cluster_resource::ClusterResource::*;

        match resource {
            Qemu(vm) => {
                vms += 1;
                println!(
                    "VM {} (ID: {}) on node {} - {}",
                    vm.common.name.as_deref().unwrap_or("(unnamed)"),
                    vm.vmid,
                    vm.common.node,
                    vm.common.status
                );
            }
            Lxc(ct) => {
                containers += 1;
                println!(
                    "Container {} (ID: {}) on node {} - {}",
                    ct.common.name.as_deref().unwrap_or("(unnamed)"),
                    ct.vmid,
                    ct.common.node,
                    ct.common.status
                );
            }
            Storage(st) => {
                storages += 1;
                println!(
                    "Storage '{}' on node {} ({} type) - {}",
                    st.storage, st.common.node, st.storage_type, st.common.status
                );
            }
            Node(node) => {
                nodes += 1;
                println!(
                    "Node {} - {} (load: {:?})",
                    node.common.node,
                    node.common.status,
                    node.loadavg.unwrap_or_default()
                );
            }
        }
    }

    println!(
        "\nSummary: {} VMs, {} containers, {} storage backends, {} nodes",
        vms, containers, storages, nodes
    );

    Ok(())
}
