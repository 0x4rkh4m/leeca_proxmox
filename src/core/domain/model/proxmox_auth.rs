use crate::core::domain::{
    error::ProxmoxResult, value_object::proxmox_csrf_token::ProxmoxCSRFToken,
    value_object::proxmox_ticket::ProxmoxTicket,
};

pub struct ProxmoxAuth {
    ticket: ProxmoxTicket,
    csrf_token: Option<ProxmoxCSRFToken>,
}

impl ProxmoxAuth {
    pub async fn new(
        ticket: ProxmoxTicket,
        csrf_token: Option<ProxmoxCSRFToken>,
    ) -> ProxmoxResult<Self> {
        Ok(Self { ticket, csrf_token })
    }

    pub fn ticket(&self) -> &ProxmoxTicket {
        &self.ticket
    }

    pub fn csrf_token(&self) -> Option<&ProxmoxCSRFToken> {
        self.csrf_token.as_ref()
    }
}
