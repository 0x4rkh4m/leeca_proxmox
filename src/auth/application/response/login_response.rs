use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginResponse {
    pub data: LoginResponseData,
}

#[derive(Deserialize)]
pub struct LoginResponseData {
    pub ticket: String,
    #[serde(rename = "CSRFPreventionToken")]
    pub csrf_token: String,
}
