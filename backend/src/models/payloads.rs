use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct AccessRequest {
    pub ips: Vec<String>,
    
    pub service: Option<String>, // "anydesk" or "teamviewer"
    pub cc_emails: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct APIResponse {
    pub status: String,
    pub message: String,
}
