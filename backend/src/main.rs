pub mod auth;
pub mod mailer;
pub mod firewall;
pub mod handlers;
pub mod models;
pub mod middleware;

use axum::{
    routing::{get, post},
    Router,
    middleware::from_fn,
};
use std::sync::Arc;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::info;
use std::env;

use crate::firewall::{FirewallProvider, MultiFirewallProvider, fortigate::FortiGateClient, paloalto::PaloAltoClient};
use crate::handlers::access::{login_handler, logout_handler, verify_handler, request_access_handler};
use crate::middleware::auth_middleware;

pub struct AppState {
    pub firewall: Box<dyn FirewallProvider>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Rate limiting configuration: 5 requests per 10 seconds per IP
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    let firewall_type = env::var("FIREWALL_TYPE").unwrap_or_else(|_| "both".to_string());
    
    let mut providers: Vec<Box<dyn FirewallProvider>> = Vec::new();
    
    if firewall_type == "fortigate" || firewall_type == "both" {
        providers.push(Box::new(FortiGateClient::new()));
    }
    
    if firewall_type == "paloalto" || firewall_type == "both" {
        providers.push(Box::new(PaloAltoClient::new()));
    }

    let firewall: Box<dyn FirewallProvider> = Box::new(MultiFirewallProvider { providers });

    let state = Arc::new(AppState { firewall });

    let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5053".to_string());
    let cors = CorsLayer::new()
        .allow_origin(cors_origin.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::HeaderName::from_static("content-type")])
        .allow_credentials(true);

    let app = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .nest("/api", 
            Router::new()
                .route("/access", post(request_access_handler))
                .route("/verify", get(verify_handler))
                .layer(from_fn(auth_middleware))
                .route("/login", post(login_handler))
                .route("/logout", post(logout_handler))
                .layer(GovernorLayer { config: governor_conf })
        )
        .layer(cors)
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "5051".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("Invalid address");
    
    info!("AnydeskAccess Backend starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
