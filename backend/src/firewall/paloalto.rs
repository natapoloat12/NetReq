use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use async_trait::async_trait;
use tracing::info;
use super::FirewallProvider;

pub struct PaloAltoClient {
    host: String,
    client: Client,
    rule_anydesk: String,
    rule_teamviewer: String,
    rule_internet: String,
    rule_update_windows: String,
    vsys: String,
}

impl PaloAltoClient {
    pub fn new() -> Self {
        let host = env::var("PALOALTO_HOST").expect("PALOALTO_HOST must be set");
        let rule_anydesk = env::var("PALOALTO_RULE_Anydesk").expect("PALOALTO_RULE_Anydesk must be set");
        let rule_teamviewer = env::var("PALOALTO_RULE_Teamview").expect("PALOALTO_RULE_Teamview must be set");
        let rule_internet = env::var("PALOALTO_RULE_Internet").expect("PALOALTO_RULE_Internet must be set");
        let rule_update_windows = env::var("PALOALTO_RULE_UpdateWindows").unwrap_or_else(|_| "T2U-Allow All MS-Update".to_string());
        let vsys = env::var("PALOALTO_VSYS").unwrap_or_else(|_| "vsys1".to_string());
        let verify_ssl = env::var("PALOALTO_VERIFY_SSL").unwrap_or_else(|_| "true".to_string()) == "true";
        
        let client = Client::builder()
            .danger_accept_invalid_certs(!verify_ssl)
            .use_rustls_tls()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            host,
            client,
            rule_anydesk,
            rule_teamviewer,
            rule_internet,
            rule_update_windows,
            vsys,
        }
    }

    async fn get_api_key(&self) -> Result<String, String> {
        if let Ok(key) = env::var("PALOALTO_API_KEY") {
            if !key.is_empty() {
                return Ok(key.trim().to_string());
            }
        }

        let user = env::var("PALOALTO_USER").map_err(|_| "PALOALTO_USER not set")?.trim().to_string();
        let pass = env::var("PALOALTO_PASSWORD").map_err(|_| "PALOALTO_PASSWORD not set")?.trim().to_string();
        
        let host_clean = self.host.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
        let url = format!("https://{}/api/", host_clean);
        let res = self.client.get(&url)
            .query(&[("type", "keygen"), ("user", &user), ("password", &pass)])
            .send()
            .await
            .map_err(|e| format!("Keygen request failed: {}", e))?;

        let text = res.text().await.map_err(|e| format!("Failed to read keygen response: {}", e))?;
        
        if let Some(start) = text.find("<key>") {
            if let Some(end) = text.find("</key>") {
                let key = &text[start + 5..end];
                return Ok(key.to_string());
            }
        }
        
        Err(format!("Could not find API key in Palo Alto response: {}", text))
    }

    async fn get_rule(&self, api_key: &str, rule_name: &str) -> Result<Value, String> {
        let host_clean = self.host.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
        let url = format!("https://{}/restapi/v10.2/Policies/SecurityRules", host_clean);
        
        let loc = "vsys".to_string();
        let out_fmt = "json".to_string();

        let res = self.client.get(&url)
            .header("X-PAN-KEY", api_key)
            .header("Accept", "application/json")
            .query(&[
                ("name", rule_name),
                ("location", &loc),
                ("vsys", &self.vsys),
                ("output-format", &out_fmt)
            ])
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Palo Alto rule: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Palo Alto GET rule failed ({}): {}", status, err_text));
        }

        let json: Value = res.json().await.map_err(|e| e.to_string())?;
        
        if let Some(entry) = json["result"]["entry"].as_array().and_then(|a| a.get(0)) {
            return Ok(entry.clone());
        } else if json["entry"].as_object().is_some() {
            return Ok(json.clone());
        }

        Err(format!("Could not find rule entry in response: {:?}", json))
    }

    async fn update_rule(&self, api_key: &str, rule_name: &str, rule_entry: Value) -> Result<(), String> {
        let host_clean = self.host.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
        let url = format!("https://{}/restapi/v10.2/Policies/SecurityRules", host_clean);
        
        let loc = "vsys".to_string();
        let in_fmt = "json".to_string();
        let out_fmt = "json".to_string();

        let payload = json!({
            "entry": rule_entry
        });

        let res = self.client.put(&url)
            .header("X-PAN-KEY", api_key)
            .header("Accept", "application/json")
            .query(&[
                ("name", rule_name),
                ("location", &loc),
                ("vsys", &self.vsys),
                ("input-format", &in_fmt),
                ("output-format", &out_fmt)
            ])
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to update Palo Alto rule: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Palo Alto PUT rule failed: {} - {}", status, err_text));
        }

        Ok(())
    }
}

#[async_trait]
impl FirewallProvider for PaloAltoClient {
    async fn add_ip_to_policy(&self, ip: &str, service: &str) -> Result<(), String> {
        let rule_name = match service.to_lowercase().as_str() {
            "teamviewer" => &self.rule_teamviewer,
            "internet" => &self.rule_internet,
            "update_windows" => &self.rule_update_windows,
            _ => &self.rule_anydesk,
        };

        info!("Palo Alto: Adding IP {} to rule {} (service: {})", ip, rule_name, service);
        
        let api_key = self.get_api_key().await?;
        let mut rule_entry = self.get_rule(&api_key, rule_name).await?;

        let source = rule_entry.get_mut("source").ok_or("Rule missing 'source' field")?;
        let members = source.get_mut("member").ok_or("Source missing 'member' field")?;
        
        let members_array = members.as_array_mut().ok_or("Source members is not an array")?;

        let ip_str = Value::String(ip.to_string());
        if members_array.contains(&ip_str) {
            info!("IP {} already exists in Palo Alto rule {}", ip, rule_name);
            return Ok(());
        }

        members_array.push(ip_str);

        self.update_rule(&api_key, rule_name, rule_entry).await?;

        info!("Successfully updated Palo Alto rule {} with IP {}", rule_name, ip);
        Ok(())
    }

    async fn commit(&self) -> Result<(), String> {
        info!("Palo Alto: Triggering commit...");
        let api_key = self.get_api_key().await?;
        
        let host_clean = self.host.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
        let url = format!("https://{}/api/", host_clean);
        
        let res = self.client.post(&url)
            .query(&[
                ("type", "commit"),
                ("cmd", "<commit></commit>"),
                ("key", &api_key)
            ])
            .send()
            .await
            .map_err(|e| format!("Failed to send commit request: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("Palo Alto commit failed ({}): {}", status, err_text));
        }

        let text = res.text().await.map_err(|e| format!("Failed to read commit response: {}", e))?;
        if text.contains("status=\"success\"") {
            info!("Palo Alto commit triggered successfully");
            Ok(())
        } else {
            Err(format!("Palo Alto commit reported failure: {}", text))
        }
    }
}
