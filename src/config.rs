use anyhow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

pub const NETX_CONFIG_TOML: &str = "config/config.toml";
pub const SING_BOX_TEMPLATE_CONFIG_JSON: &str = "config/config.json.template";
pub const SING_BOX_CONFIG_JSON: &str = "config/config.json";

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u8,
    pub terminal: String,
    pub singbox: String,
    pub log: LogConfig,
    pub profiles: Vec<Profile>,
}

#[derive(Serialize, Deserialize)]
pub struct LogConfig {
    pub level: LogLevel,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub uuid: String,
    pub r#type: ProfileType,
    pub server: String,
    pub server_port: u16,
    pub tls: TLS,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProfileType {
    Vless,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TLS {
    pub server_name: String,
    pub utls: UTLS,
    pub reality: Reality,
    pub transport: Transport,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UTLS {
    pub fingerprint: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Reality {
    pub public_key: String,
    pub short_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transport {
    pub service_name: String,
}

impl Default for Config {
    #[cfg(target_os = "linux")]
    fn default() -> Self {
        Self {
            version: 1,
            terminal: "alacritty".to_string(),
            singbox: "/usr/bin/sing-box".to_string(),
            log: LogConfig {
                level: LogLevel::Warn,
            },
            profiles: vec![],
        }
    }

    #[cfg(target_os = "windows")]
    fn default() -> Self {
        Self {
            version: 1,
            terminal: "cmd".to_string(),
            singbox: "C:\\Program Files\\sing-box\\sing-box.exe".to_string(),
            log: LogConfig {
                level: LogLevel::Warn,
            },
            profiles: vec![],
        }
    }
}

pub struct ConfigManager {}

impl ConfigManager {
    pub fn load_config() -> anyhow::Result<Config> {
        let config_str = fs::read_to_string(NETX_CONFIG_TOML);

        match config_str {
            Ok(str) => {
                if str.trim().is_empty() {
                    let default_config = Config::default();
                    Self::write_config(&default_config)?;
                    return Ok(default_config);
                }
                Ok(toml::from_str(&str)?)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let default_config = Config::default();
                Self::write_config(&default_config)?;
                Ok(default_config)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn write_config(config: &Config) -> anyhow::Result<()> {
        let toml_data = toml::to_string_pretty(&config)?;

        if let Some(parent) = std::path::Path::new(NETX_CONFIG_TOML).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::File::create(NETX_CONFIG_TOML)?.write_all(toml_data.as_bytes())?;
        Ok(())
    }

    pub fn add_profile(profile: Profile) {
        let mut config = ConfigManager::load_config().unwrap();
        config.profiles.append(&mut vec![profile]);
        ConfigManager::write_config(&config).unwrap();
    }
    pub fn delete_profile(index: usize) {
        let mut config = ConfigManager::load_config().unwrap();
        config.profiles.remove(index);
        ConfigManager::write_config(&config).unwrap();
    }
}
