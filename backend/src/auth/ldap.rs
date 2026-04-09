use ldap3::{LdapConnAsync, LdapConnSettings, SearchEntry, Scope};
use std::env;
use tracing::{info, warn};

pub struct LdapAuthResult {
    pub username: String,
    pub email: Option<String>,
    pub fullname: Option<String>,
    pub employee_id: Option<String>,
}

fn sanitize_ldap_input(input: &str) -> String {
    input.replace('\\', "\\5c")
         .replace('*', "\\2a")
         .replace('(', "\\28")
         .replace(')', "\\29")
         .replace('\0', "\\00")
}

pub async fn authenticate_with_ldap(username: &str, password: &str) -> Result<LdapAuthResult, String> {
    let ldap_url = env::var("LDAP_URL").map_err(|_| "LDAP_URL not set")?;
    
    let clean_username = if let Some(pos) = username.find('\\') {
        &username[pos + 1..]
    } else {
        username
    };

    let upn_co_th = format!("{}@kce.co.th", clean_username);
    let upn_local = format!("{}@kce.local", clean_username);

    let settings = LdapConnSettings::new()
        .set_conn_timeout(std::time::Duration::from_secs(5));

    let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &ldap_url)
        .await
        .map_err(|e| {
            warn!("LDAP connection failure: {}", e);
            "Authentication service unavailable".to_string()
        })?;

    ldap3::drive!(conn);

    info!("LDAP: Attempting bind for {}...", upn_co_th);
    let mut bind_res = ldap.simple_bind(&upn_co_th, password).await;
    let mut authenticated = match bind_res {
        Ok(res) => {
            if res.clone().success().is_ok() {
                info!("LDAP: Bind SUCCESS for {}", upn_co_th);
                true
            } else {
                warn!("LDAP: Bind FAILED for {}: {:?}", upn_co_th, res);
                false
            }
        },
        Err(e) => {
            warn!("LDAP: Bind ERROR for {}: {}", upn_co_th, e);
            false
        }
    };

    if !authenticated {
        info!("LDAP: Attempting bind for {}...", upn_local);
        bind_res = ldap.simple_bind(&upn_local, password).await;
        authenticated = match bind_res {
            Ok(res) => {
                if res.clone().success().is_ok() {
                    info!("LDAP: Bind SUCCESS for {}", upn_local);
                    true
                } else {
                    warn!("LDAP: Bind FAILED for {}: {:?}", upn_local, res);
                    false
                }
            },
            Err(e) => {
                warn!("LDAP: Bind ERROR for {}: {}", upn_local, e);
                false
            }
        };
    }

    if !authenticated {
        warn!("LDAP: All bind attempts failed for user {}", clean_username);
        return Err("Invalid credentials".to_string());
    }

    let mut user_email = None;
    let mut fullname = None;
    let mut employee_id = None;
    let mut is_member_of_required_group = false;
    let required_group = "G-KCE-IT-SI";

    let safe_username = sanitize_ldap_input(clean_username);
    let search_filter = format!("(|(userPrincipalName={})(userPrincipalName={})(sAMAccountName={}))", 
        upn_co_th, upn_local, safe_username);
    
    let attrs = vec!["mail", "displayName", "employeeID", "memberOf"];

    if let Ok(search_res) = ldap.search("DC=kce,DC=co,DC=th", Scope::Subtree, &search_filter, attrs.clone()).await {
        if let Ok((results, _)) = search_res.success() {
            if !results.is_empty() {
                let entry = SearchEntry::construct(results[0].clone());
                user_email = entry.attrs.get("mail").and_then(|m| m.get(0).cloned());
                fullname = entry.attrs.get("displayName").and_then(|m| m.get(0).cloned());
                employee_id = entry.attrs.get("employeeID").and_then(|m| m.get(0).cloned());
                
                if let Some(member_of) = entry.attrs.get("memberOf") {
                    is_member_of_required_group = member_of.iter().any(|group_dn| {
                        group_dn.to_uppercase().contains(&format!("CN={},", required_group.to_uppercase())) ||
                        group_dn.to_uppercase().ends_with(&format!("CN={}", required_group.to_uppercase()))
                    });
                }
            }
        }
    }

    if !is_member_of_required_group {
        if let Ok(search_res) = ldap.search("DC=kce,DC=local", Scope::Subtree, &search_filter, attrs).await {
            if let Ok((results, _)) = search_res.success() {
                if !results.is_empty() {
                    let entry = SearchEntry::construct(results[0].clone());
                    user_email = entry.attrs.get("mail").and_then(|m| m.get(0).cloned());
                    fullname = entry.attrs.get("displayName").and_then(|m| m.get(0).cloned());
                    employee_id = entry.attrs.get("employeeID").and_then(|m| m.get(0).cloned());
                    
                    if let Some(member_of) = entry.attrs.get("memberOf") {
                        is_member_of_required_group = member_of.iter().any(|group_dn| {
                            group_dn.to_uppercase().contains(&format!("CN={},", required_group.to_uppercase())) ||
                            group_dn.to_uppercase().ends_with(&format!("CN={}", required_group.to_uppercase()))
                        });
                    }
                }
            }
        }
    }

    if !is_member_of_required_group {
        warn!("LDAP: User {} is not a member of the required group {}", clean_username, required_group);
        return Err(format!("Access denied: User must be a member of group {}", required_group));
    }

    Ok(LdapAuthResult {
        username: clean_username.to_string(),
        email: user_email,
        fullname,
        employee_id,
    })
}
