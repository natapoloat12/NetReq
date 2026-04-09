use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use chrono::{Utc, Duration};

pub const AUTH_COOKIE_NAME: &str = "anydesk_jwt_auth";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub fullname: Option<String>,
    pub employee_id: Option<String>,
    pub exp: i64,
}

pub fn create_jwt(sub: String, email: Option<String>, fullname: Option<String>, employee_id: Option<String>) -> Result<String, String> {
    let secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET not set")?;
    let exp = (Utc::now() + Duration::hours(8)).timestamp();
    
    let claims = Claims {
        sub,
        email,
        fullname,
        employee_id,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ).map_err(|e| e.to_string())
}

pub fn validate_jwt(token: &str) -> Result<Claims, String> {
    let secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET not set")?;
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("JWT Validation failed: {}", e))
}
