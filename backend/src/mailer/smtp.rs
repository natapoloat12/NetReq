use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;
use std::collections::HashSet;
use tracing::{info, error, warn};

pub struct Mailer;

impl Mailer {
    pub async fn send_access_notification(
        user_email: &str,
        ips: &[String],
        service: &str,
        cc_emails: Option<Vec<String>>,
        requester_name: &str,
    ) {
        let smtp_host = env::var("SMTP_HOST").ok();
        let smtp_port = env::var("SMTP_PORT").ok().and_then(|p| p.parse::<u16>().ok());
        let smtp_user = env::var("SMTP_USER").ok();
        let smtp_pass = env::var("SMTP_PASS").ok();
        let smtp_from = env::var("SMTP_FROM").ok();
        let smtp_to = env::var("SMTP_TO").ok();
        let smtp_cc = env::var("SMTP_CC").ok();

        if let (Some(host), Some(port), Some(user), Some(pass), Some(from_email)) = 
               (smtp_host, smtp_port, smtp_user, smtp_pass, smtp_from) {
            
            let mut builder = Message::builder()
                .from(from_email.parse().expect("Invalid SMTP_FROM"));

            let mut to_set = HashSet::new();
            to_set.insert(user_email.to_string());
            if let Some(admin_to) = smtp_to {
                for email in admin_to.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                    to_set.insert(email.to_string());
                }
            }

            for email in &to_set {
                if let Ok(mailbox) = email.parse() {
                    builder = builder.to(mailbox);
                }
            }

            let mut cc_set = HashSet::new();
            if let Some(env_cc) = smtp_cc {
                for email in env_cc.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                    cc_set.insert(email.to_string());
                }
            }
            if let Some(req_cc) = cc_emails {
                for email in req_cc {
                    cc_set.insert(email.trim().to_string());
                }
            }

            for email in &cc_set {
                if !to_set.contains(email) {
                    if let Ok(mailbox) = email.parse() {
                        builder = builder.cc(mailbox);
                    }
                }
            }

            let ips_str = ips.join(", ");
            let ips_html = ips.iter()
                .map(|ip| format!("<tr><td style='border: 1px solid #ddd; padding: 8px; text-align: center;'>{}</td><td style='border: 1px solid #ddd; padding: 8px; text-align: center;'>{}</td><td style='border: 1px solid #ddd; padding: 8px; text-align: center;'>{}</td><td style='border: 1px solid #ddd; padding: 8px; text-align: center;'>Granted</td></tr>", requester_name, ip, service))
                .collect::<Vec<_>>()
                .join("");

            let email_html = format!(
                "<!DOCTYPE html><html><body style='font-family: sans-serif;'>\
                    <h3>Internet Access Request Notification</h3>\
                    <p>Access has been granted for the following details:</p>\
                    <table style='border-collapse: collapse; width: 100%;'>\
                        <tr style='background-color: #f2f2f2;'>\
                            <th style='border: 1px solid #ddd; padding: 8px;'>Requester</th>\
                            <th style='border: 1px solid #ddd; padding: 8px;'>IP Address</th>\
                            <th style='border: 1px solid #ddd; padding: 8px;'>Service</th>\
                            <th style='border: 1px solid #ddd; padding: 8px;'>Status</th>\
                        </tr>\
                        {}\
                    </table>\
                    <p style='color: #777; font-size: 0.8em; margin-top: 20px;'>\
                        System: Internet Access Portal\
                    </p>\
                </body></html>",
                ips_html
            );

            let email = builder
                .subject(format!("Internet Access Granted: {} ({})", ips_str, service))
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(email_html)
                .unwrap();

            let creds = Credentials::new(user, pass);
            let tls_params = TlsParameters::new(host.clone()).expect("Invalid TLS parameters");

            let mailer: AsyncSmtpTransport<Tokio1Executor> = if port == 465 {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
                    .unwrap()
                    .port(port)
                    .tls(Tls::Wrapper(tls_params))
                    .credentials(creds)
                    .build()
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
                    .unwrap()
                    .port(port)
                    .tls(Tls::Required(tls_params))
                    .credentials(creds)
                    .build()
            };

            match mailer.send(email).await {
                Ok(_) => info!("Access notification email sent for IPs: {}", ips_str),
                Err(e) => error!("Failed to send access notification: {}", e),
            }
        } else {
            warn!("SMTP configuration incomplete; cannot send email.");
        }
    }
}
