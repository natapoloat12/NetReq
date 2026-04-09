use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use async_trait::async_trait;
use tracing::{info, error};
use super::FirewallProvider;

pub struct FortiGateClient {
    base_url: String,
    api_token: String,
    client: Client,
    policy_ids_anydesk: Vec<String>,
    policy_ids_teamviewer: Vec<String>,
}

impl FortiGateClient {
    pub fn new() -> Self {
        let base_url = env::var("FORTIGATE_BASE_URL").expect("FORTIGATE_BASE_URL must be set");
        let api_token = env::var("FORTIGATE_API_TOKEN").expect("FORTIGATE_API_TOKEN must be set");
        
        let anydesk_raw = env::var("FORTIGATE_POLICY_IDS_ANYDESK").unwrap_or_else(|_| "135,12".to_string());
        let policy_ids_anydesk = anydesk_raw.split(',').map(|s| s.trim().to_string()).collect();

        let teamviewer_raw = env::var("FORTIGATE_POLICY_IDS_TEAMVIEWER").unwrap_or_else(|_| "135,12".to_string());
        let policy_ids_teamviewer = teamviewer_raw.split(',').map(|s| s.trim().to_string()).collect();

        let verify_ssl = env::var("FORTIGATE_VERIFY_SSL").unwrap_or_else(|_| "true".to_string()) == "true";
        
        let client = Client::builder()
            .danger_accept_invalid_certs(!verify_ssl)
            .use_rustls_tls()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            api_token,
            client,
            policy_ids_anydesk,
            policy_ids_teamviewer,
        }
    }

    async fn ensure_address_object(&self, ip: &str) -> Result<String, String> {
        let filter = format!("subnet=={} 255.255.255.255", ip);
        let search_url = format!("{}/api/v2/cmdb/firewall/address?filter={}", self.base_url, url_escape::encode_component(&filter));
        
        let res_ip = self.client.get(&search_url)
            .bearer_auth(&self.api_token)
            .send()
            .await
            .map_err(|e| format!("Search address object failed: {}", e))?;

        if res_ip.status().is_success() {
            let json: Value = res_ip.json().await.map_err(|e| format!("Failed to parse search results: {}", e))?;
            if let Some(results) = json["results"].as_array() {
                if !results.is_empty() {
                    let existing_name = results[0]["name"].as_str().ok_or("Missing address name")?;
                    info!("Found existing address object '{}' for IP {}", existing_name, ip);
                    return Ok(existing_name.to_string());
                }
            }
        }

        let name = format!("ADDR_{}", ip.replace('.', "_"));
        let name_url = format!("{}/api/v2/cmdb/firewall/address/{}", self.base_url, url_escape::encode_component(&name));
        let res_name = self.client.get(&name_url)
            .bearer_auth(&self.api_token)
            .send()
            .await;

        if let Ok(response) = res_name {
            if response.status().is_success() {
                info!("Found existing address object by name: {}", name);
                return Ok(name);
            }
        }

        let create_url = format!("{}/api/v2/cmdb/firewall/address", self.base_url);
        let payload = json!({
            "name": name,
            "type": "ipmask",
            "subnet": format!("{}/32", ip),
            "comment": "Created via Anydesk Access Portal",
        });

        let response = self.client.post(&create_url)
            .bearer_auth(&self.api_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to create address: {}", e))?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            if !err_text.contains("already exists") {
                return Err(format!("FortiGate Error (Address): {}", err_text));
            }
        }
        
        info!("Created new address object: {}", name);
        Ok(name)
    }
}

#[async_trait]
impl FirewallProvider for FortiGateClient {
    async fn add_ip_to_policy(&self, ip: &str, service: &str) -> Result<(), String> {
        if service.to_lowercase() == "internet" {
            info!("FortiGate: Skipping operations for service '{}'", service);
            return Ok(());
        }

        let addr_name = self.ensure_address_object(ip).await?;
        
        let policy_ids = if service.to_lowercase() == "teamviewer" {
            &self.policy_ids_teamviewer
        } else {
            &self.policy_ids_anydesk
        };

        for policy_id in policy_ids {
            info!("FortiGate: Appending IP {} (as {}) to policy {} (service: {})", ip, addr_name, policy_id, service);
            
            let url = format!("{}/api/v2/cmdb/firewall/policy/{}", self.base_url, policy_id);
            let response = self.client.get(&url)
                .bearer_auth(&self.api_token)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch policy: {}", e))?;

            if !response.status().is_success() {
                error!("Failed to fetch policy {}: {}", policy_id, response.status());
                continue; 
            }

            let json: Value = response.json().await.map_err(|e| e.to_string())?;
            let mut srcaddr = json["results"][0]["srcaddr"].as_array().cloned().ok_or("Failed to parse srcaddr")?;

            if srcaddr.iter().any(|a| a["name"] == addr_name) {
                info!("IP {} is already in policy {}", ip, policy_id);
                continue;
            }

            srcaddr.push(json!({"name": addr_name}));
            let update_payload = json!({ "srcaddr": srcaddr });

            let update_res = self.client.put(&url)
                .bearer_auth(&self.api_token)
                .json(&update_payload)
                .send()
                .await
                .map_err(|e| format!("Failed to update policy: {}", e))?;

            if !update_res.status().is_success() {
                let err = update_res.text().await.unwrap_or_default();
                error!("Failed to update policy {}: {}", policy_id, err);
                continue;
            }

            info!("Successfully added IP {} to policy {}", ip, policy_id);
        }
        
        Ok(())
    }

    async fn commit(&self) -> Result<(), String> {
        // FortiGate does not require a commit operation
        Ok(())
    }
}
