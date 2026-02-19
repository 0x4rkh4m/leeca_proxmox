use crate::core::domain::value_object::{ProxmoxCSRFToken, ProxmoxTicket};

#[derive(Debug, Clone)]
pub struct ProxmoxAuth {
    ticket: ProxmoxTicket,
    csrf_token: Option<ProxmoxCSRFToken>,
}

impl ProxmoxAuth {
    pub fn new(ticket: ProxmoxTicket, csrf_token: Option<ProxmoxCSRFToken>) -> Self {
        Self { ticket, csrf_token }
    }

    pub fn ticket(&self) -> &ProxmoxTicket {
        &self.ticket
    }

    pub fn csrf_token(&self) -> Option<&ProxmoxCSRFToken> {
        self.csrf_token.as_ref()
    }
}
