mod proxmox_csrf_token;
mod proxmox_host;
mod proxmox_password;
mod proxmox_port;
mod proxmox_realm;
mod proxmox_ticket;
mod proxmox_uri;
mod proxmox_username;

pub use proxmox_csrf_token::ProxmoxCSRFToken;
pub use proxmox_host::ProxmoxHost;
pub use proxmox_password::ProxmoxPassword;
pub use proxmox_port::ProxmoxPort;
pub use proxmox_realm::ProxmoxRealm;
pub use proxmox_ticket::ProxmoxTicket;
pub use proxmox_uri::ProxmoxUrl;
pub use proxmox_username::ProxmoxUsername;

// Re-export validation functions for internal use
pub(crate) use proxmox_csrf_token::validate_csrf_token;
pub(crate) use proxmox_host::validate_host;
pub(crate) use proxmox_password::validate_password;
pub(crate) use proxmox_port::validate_port;
pub(crate) use proxmox_realm::validate_realm;
pub(crate) use proxmox_ticket::validate_ticket;
pub(crate) use proxmox_uri::validate_url;
pub(crate) use proxmox_username::validate_username;
