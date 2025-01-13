use serde::Serialize;

#[derive(Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub realm: String,
}
