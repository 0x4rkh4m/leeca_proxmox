use crate::core::domain::{
    error::ProxmoxResult, value_object::proxmox_host::ProxmoxHost,
    value_object::proxmox_password::ProxmoxPassword, value_object::proxmox_port::ProxmoxPort,
    value_object::proxmox_realm::ProxmoxRealm, value_object::proxmox_uri::ProxmoxUrl,
    value_object::proxmox_username::ProxmoxUsername,
};

pub struct ProxmoxConnection {
    proxmox_host: ProxmoxHost,
    proxmox_port: ProxmoxPort,
    proxmox_username: ProxmoxUsername,
    proxmox_password: ProxmoxPassword,
    proxmox_realm: ProxmoxRealm,
    proxmox_secure: bool, // TODO: Make this a value object (or force only https)
    proxmox_url: ProxmoxUrl,
}

impl ProxmoxConnection {
    pub async fn new(
        proxmox_host: ProxmoxHost,
        proxmox_port: ProxmoxPort,
        proxmox_username: ProxmoxUsername,
        proxmox_password: ProxmoxPassword,
        proxmox_realm: ProxmoxRealm,
        proxmox_secure: bool,
    ) -> ProxmoxResult<Self> {
        let url = ProxmoxUrl::new(&proxmox_host, &proxmox_port, &proxmox_secure).await?;
        Ok(Self {
            proxmox_host,
            proxmox_port,
            proxmox_username,
            proxmox_password,
            proxmox_realm,
            proxmox_secure,
            proxmox_url: url,
        })
    }

    pub fn proxmox_host(&self) -> &ProxmoxHost {
        &self.proxmox_host
    }

    pub fn proxmox_port(&self) -> &ProxmoxPort {
        &self.proxmox_port
    }

    pub fn proxmox_username(&self) -> &ProxmoxUsername {
        &self.proxmox_username
    }

    pub fn proxmox_password(&self) -> &ProxmoxPassword {
        &self.proxmox_password
    }

    pub fn proxmox_realm(&self) -> &ProxmoxRealm {
        &self.proxmox_realm
    }

    pub fn is_connection_secure(&self) -> &bool {
        &self.proxmox_secure
    }

    pub fn proxmox_url(&self) -> &ProxmoxUrl {
        &self.proxmox_url
    }
}
