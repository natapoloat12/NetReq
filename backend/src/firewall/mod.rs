pub mod fortigate;
pub mod paloalto;

use async_trait::async_trait;

#[async_trait]
pub trait FirewallProvider: Send + Sync {
    async fn add_ip_to_policy(&self, ip: &str, service: &str) -> Result<(), String>;
    async fn commit(&self) -> Result<(), String>;
}

pub struct MultiFirewallProvider {
    pub providers: Vec<Box<dyn FirewallProvider>>,
}

#[async_trait]
impl FirewallProvider for MultiFirewallProvider {
    async fn add_ip_to_policy(&self, ip: &str, service: &str) -> Result<(), String> {
        let mut errors = Vec::new();
        for provider in &self.providers {
            if let Err(e) = provider.add_ip_to_policy(ip, service).await {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join(" | "))
        }
    }

    async fn commit(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        for provider in &self.providers {
            if let Err(e) = provider.commit().await {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join(" | "))
        }
    }
}
