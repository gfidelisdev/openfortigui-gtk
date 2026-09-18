use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VpnProfile {
    pub name: String,
    pub gateway_host: String,
    pub gateway_port: u16,
    pub username: String,
    pub password: String,
    pub realm: String,
    pub trusted_cert: String,
    pub saml_login: bool,
    pub saml_port: u16,
    pub set_routes: bool,
    pub set_dns: bool,
    pub persistent: bool,
}

impl Default for VpnProfile {
    fn default() -> Self {
        Self {
            name: "Novo perfil".to_string(),
            gateway_host: String::new(),
            gateway_port: 443,
            username: String::new(),
            password: String::new(),
            realm: String::new(),
            trusted_cert: String::new(),
            saml_login: false,
            saml_port: 8020,
            set_routes: true,
            set_dns: true,
            persistent: false,
        }
    }
}

pub fn load_profiles() -> Vec<VpnProfile> {
    let path = profiles_path();
    let Ok(data) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_profiles(profiles: &[VpnProfile]) -> Result<(), Box<dyn std::error::Error>> {
    let path = profiles_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(profiles)?)?;
    Ok(())
}

fn profiles_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openfortigui-gtk")
        .join("profiles.json")
}

pub fn unique_profile(profiles: &[VpnProfile]) -> VpnProfile {
    let mut profile = VpnProfile::default();
    if profiles.iter().all(|existing| existing.name != profile.name) {
        return profile;
    }

    for number in 2.. {
        let candidate = format!("Novo perfil {number}");
        if profiles.iter().all(|existing| existing.name != candidate) {
            profile.name = candidate;
            return profile;
        }
    }
    profile
}

/// Writes the openfortivpn config consumed by the privileged helper; kept 0600 since it may hold a plaintext password.
pub fn write_runtime_config(profile: &VpnProfile) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let directory = dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("openfortigui-gtk");
    fs::create_dir_all(&directory)?;

    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = directory.join(format!("openfortivpn-{}-{unique}.conf", std::process::id()));

    let mut config = String::new();
    push_config(&mut config, "host", profile.gateway_host.trim());
    push_config(&mut config, "port", &profile.gateway_port.to_string());

    if !profile.saml_login {
        push_config(&mut config, "username", profile.username.trim());
        if !profile.password.is_empty() {
            push_config(&mut config, "password", &profile.password);
        }
    }
    if !profile.realm.trim().is_empty() {
        push_config(&mut config, "realm", profile.realm.trim());
    }
    if !profile.trusted_cert.trim().is_empty() {
        push_config(&mut config, "trusted-cert", profile.trusted_cert.trim());
    }
    if profile.saml_login {
        push_config(&mut config, "saml-login", &profile.saml_port.to_string());
    }
    push_config(&mut config, "set-routes", if profile.set_routes { "1" } else { "0" });
    push_config(&mut config, "set-dns", if profile.set_dns { "1" } else { "0" });
    if profile.persistent {
        push_config(&mut config, "persistent", "1");
    }

    fs::write(&path, config)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(path)
}

fn push_config(config: &mut String, key: &str, value: &str) {
    if !value.is_empty() {
        config.push_str(key);
        config.push_str(" = ");
        config.push_str(value);
        config.push('\n');
    }
}
