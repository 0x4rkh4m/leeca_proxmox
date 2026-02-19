use crate::core::domain::value_object::{
    ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm, ProxmoxUrl, ProxmoxUsername,
};

/// Connection details for a Proxmox server.
///
/// This struct holds all validated configuration needed to connect.
/// Fields are private; access is via getters.
#[derive(Debug, Clone)]
pub struct ProxmoxConnection {
    host: ProxmoxHost,
    port: ProxmoxPort,
    username: ProxmoxUsername,
    password: ProxmoxPassword,
    realm: ProxmoxRealm,
    secure: bool,
    accept_invalid_certs: bool,
    url: ProxmoxUrl,
}

#[allow(clippy::too_many_arguments)]
impl ProxmoxConnection {
    /// Creates a new connection instance (internal use only, values are already validated).
    pub(crate) fn new(
        host: ProxmoxHost,
        port: ProxmoxPort,
        username: ProxmoxUsername,
        password: ProxmoxPassword,
        realm: ProxmoxRealm,
        secure: bool,
        accept_invalid_certs: bool,
        url: ProxmoxUrl,
    ) -> Self {
        Self {
            host,
            port,
            username,
            password,
            realm,
            secure,
            accept_invalid_certs,
            url,
        }
    }

    /// Returns the host.
    #[allow(dead_code)] // Part of public API, not yet used internally.
    pub fn host(&self) -> &ProxmoxHost {
        &self.host
    }

    /// Returns the port.
    #[allow(dead_code)] // Part of public API, not yet used internally.
    pub fn port(&self) -> &ProxmoxPort {
        &self.port
    }

    /// Returns the username.
    pub fn username(&self) -> &ProxmoxUsername {
        &self.username
    }

    /// Returns the password.
    pub fn password(&self) -> &ProxmoxPassword {
        &self.password
    }

    /// Returns the realm.
    pub fn realm(&self) -> &ProxmoxRealm {
        &self.realm
    }

    /// Returns whether HTTPS is used.
    #[allow(dead_code)] // Part of public API, not yet used internally.
    pub fn is_secure(&self) -> bool {
        self.secure
    }

    /// Returns whether invalid certificates are accepted.
    pub fn accept_invalid_certs(&self) -> bool {
        self.accept_invalid_certs
    }

    /// Returns the base URL.
    pub fn url(&self) -> &ProxmoxUrl {
        &self.url
    }
}
