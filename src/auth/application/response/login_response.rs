use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginResponse {
    pub ticket: String,
    #[serde(rename = "CSRFPreventionToken")]
    pub csrf_token: String,
}
