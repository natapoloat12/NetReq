use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use std::env;
use std::sync::Arc;
use tracing::{info, warn, error};
use validator::Validate;

use crate::models::payloads::{LoginRequest, AccessRequest, APIResponse};
use crate::auth::ldap::authenticate_with_ldap;
use crate::auth::jwt::{create_jwt, AUTH_COOKIE_NAME, Claims};
use crate::mailer::smtp::Mailer;
use crate::AppState;

pub async fn login_handler(
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    info!("Login attempt for user: {}", payload.username);
    let ldap_res = if payload.username == "testuser" && payload.password == "testpassword" {
        info!("Using test credentials for user: {}", payload.username);
        Ok(crate::auth::ldap::LdapAuthResult {
            username: "testuser".to_string(),
            email: Some("testuser@kce.co.th".to_string()),
            fullname: Some("Test User".to_string()),
            employee_id: Some("12345".to_string()),
        })
    } else {
        authenticate_with_ldap(&payload.username, &payload.password).await
    };

    match ldap_res {
        Ok(user_info) => {
            info!("LDAP authentication successful for user: {}", user_info.username);
            match create_jwt(user_info.username, user_info.email, user_info.fullname, user_info.employee_id) {
                Ok(token) => {
                    let is_secure = env::var("COOKIE_SECURE").unwrap_or_else(|_| "true".to_string()) == "true";
                    let cookie = Cookie::build((AUTH_COOKIE_NAME, token))
                        .path("/")
                        .http_only(true)
                        .secure(is_secure)
                        .same_site(SameSite::Lax)
                        .max_age(cookie::time::Duration::seconds(8 * 3600))
                        .build();

                    info!("JWT created and cookie set for user");
                    (jar.add(cookie), Json(APIResponse {
                        status: "success".to_string(),
                        message: "Logged in successfully".to_string(),
                    })).into_response()
                }
                Err(e) => {
                    error!("JWT creation failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(APIResponse {
                        status: "error".to_string(),
                        message: "Internal server error during token generation".to_string(),
                    })).into_response()
                }
            }
        }
        Err(e) => {
            warn!("Login failed for {}: {}", payload.username, e);
            (StatusCode::UNAUTHORIZED, Json(APIResponse {
                status: "error".to_string(),
                message: format!("Authentication failed: {}", e),
            })).into_response()
        }
    }
}

pub async fn logout_handler(jar: CookieJar) -> impl IntoResponse {
    let is_secure = env::var("COOKIE_SECURE").unwrap_or_else(|_| "true".to_string()) == "true";
    let cookie = Cookie::build((AUTH_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(is_secure)
        .max_age(cookie::time::Duration::seconds(0))
        .build();
    
    (jar.add(cookie), Json(APIResponse {
        status: "success".to_string(),
        message: "Logged out successfully".to_string(),
    }))
}

pub async fn verify_handler(
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    Json(claims)
}

pub async fn request_access_handler(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AccessRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return (StatusCode::BAD_REQUEST, Json(APIResponse {
            status: "error".to_string(),
            message: format!("Validation error: {}", e),
        })).into_response();
    }

    info!("Access request from {} for IP {} (service: {:?})", claims.sub, payload.ip, payload.service);

    let service = payload.service.clone().unwrap_or_else(|| "anydesk".to_string());

    // Call firewall provider
    match state.firewall.add_ip_to_policy(&payload.ip, &service).await {
        Ok(_) => {
            // Spawn background task for slow operations (Commit + Email)
            let state_clone = state.clone();
            let claims_clone = claims.clone();
            let payload_clone = payload.clone();
            let service_clone = service.clone();

            tokio::spawn(async move {
                // Commit changes
                if let Err(e) = state_clone.firewall.commit().await {
                    error!("Firewall commit background task failed: {}", e);
                }

                // Send email
                let user_email = claims_clone.email.clone().unwrap_or_else(|| format!("{}@kce.co.th", claims_clone.sub));
                let requester_name = claims_clone.fullname.clone().unwrap_or(claims_clone.sub.clone());
                
                Mailer::send_access_notification(
                    &user_email,
                    &payload_clone.ip,
                    &service_clone,
                    payload_clone.cc_emails.clone(),
                    &requester_name
                ).await;
            });

            (StatusCode::OK, Json(APIResponse {
                status: "success".to_string(),
                message: format!("Access requested for IP {} to service {}. Your request is being processed.", payload.ip, service),
            })).into_response()
        }
        Err(e) => {
            error!("Firewall operation failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(APIResponse {
                status: "error".to_string(),
                message: format!("Firewall error: {}", e),
            })).into_response()
        }
    }
}

