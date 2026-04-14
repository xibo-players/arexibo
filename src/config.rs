// Xibo player Rust implementation, (c) 2022-2024 Georg Brandl.
// Licensed under the GNU AGPL, version 3 or later.

//! Definitions for the player configuration.

use std::{collections::HashMap, fs::File, path::Path, time::Duration};
use anyhow::{Context, Result};
use md5::{Md5, Digest};
use serde::{Serialize, Deserialize};
use crate::command::Command;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlayerSettings {
    #[serde(default = "default_collect_interval")]
    pub collect_interval: u64,
    #[serde(default)]
    pub stats_enabled: bool,
    #[serde(default)]
    pub xmr_network_address: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub screenshot_interval: u64,
    #[serde(default = "default_embedded_server_port")]
    pub embedded_server_port: u16,
    #[serde(default)]
    pub prevent_sleep: bool,
    #[serde(default = "default_display_name")]
    pub display_name: String,
    #[serde(default)]
    pub size_x: i32,
    #[serde(default)]
    pub size_y: i32,
    #[serde(default)]
    pub pos_x: i32,
    #[serde(default)]
    pub pos_y: i32,
    #[serde(default)]
    pub commands: HashMap<String, Command>,
}

impl PlayerSettings {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        serde_json::from_reader(File::open(path.as_ref())?)
            .context("deserializing player settings")
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        serde_json::to_writer_pretty(File::create(path.as_ref())?, self)
            .context("serializing player settings")
    }
}

fn default_collect_interval() -> u64 { 900 }
fn default_log_level() -> String { "debug".into() }
fn default_embedded_server_port() -> u16 { 9696 }
fn default_display_name() -> String { "Xibo".into() }

#[derive(Debug, Serialize, Deserialize)]
pub struct CmsSettings {
    pub address: String,
    pub key: String,
    pub display_id: String,
    pub display_name: Option<String>,
    pub proxy: Option<String>,
}

impl CmsSettings {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        serde_json::from_reader(File::open(path.as_ref())?)
            .context("deserializing player settings")
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        serde_json::to_writer_pretty(File::create(path.as_ref())?, self)
            .context("serializing player settings")
    }

    pub fn xmr_channel(&self) -> String {
        let to_hash = format!("{}{}{}", self.address, self.key, self.display_id);
        hex::encode(Md5::digest(to_hash))
    }

    /// Deterministic XMR channel ID: `MD5(address + key + display_id)`
    pub fn make_agent(&self, no_verify: bool) -> Result<ureq::Agent> {
        let tls_config = ureq::tls::TlsConfig::builder()
            .disable_verification(no_verify)
            .build();
        let proxy = if let Some(proxy) = &self.proxy {
            Some(ureq::Proxy::new(proxy)?)
        } else {
            None
        };
        Ok(ureq::config::Config::builder()
            .timeout_connect(Some(Duration::from_secs(3)))
            .tls_config(tls_config)
            .proxy(proxy)
            .build().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_settings_defaults() {
        let s: PlayerSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(s.collect_interval, 900);
        assert_eq!(s.log_level, "debug");
        assert_eq!(s.embedded_server_port, 9696);
        assert_eq!(s.display_name, "Xibo");
        assert!(!s.stats_enabled);
        assert!(!s.prevent_sleep);
    }

    #[test]
    fn player_settings_custom_values() {
        let json = r#"{"collect_interval": 60, "stats_enabled": true, "display_name": "Lobby"}"#;
        let s: PlayerSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.collect_interval, 60);
        assert!(s.stats_enabled);
        assert_eq!(s.display_name, "Lobby");
        // defaults for unspecified fields
        assert_eq!(s.log_level, "debug");
    }

    #[test]
    fn player_settings_roundtrip() {
        let original = PlayerSettings {
            collect_interval: 300,
            stats_enabled: true,
            log_level: "info".into(),
            display_name: "Test Display".into(),
            ..Default::default()
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: PlayerSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn cms_settings_xmr_channel_deterministic() {
        let cms = CmsSettings {
            address: "https://cms.example.com".into(),
            key: "secret123".into(),
            display_id: "abc-def".into(),
            display_name: None,
            proxy: None,
        };
        let ch1 = cms.xmr_channel();
        let ch2 = cms.xmr_channel();
        assert_eq!(ch1, ch2);
        assert_eq!(ch1.len(), 32); // MD5 hex = 32 chars
    }

    #[test]
    fn cms_settings_xmr_channel_varies() {
        let cms1 = CmsSettings {
            address: "https://a.com".into(),
            key: "key1".into(),
            display_id: "d1".into(),
            display_name: None,
            proxy: None,
        };
        let cms2 = CmsSettings {
            address: "https://b.com".into(),
            key: "key1".into(),
            display_id: "d1".into(),
            display_name: None,
            proxy: None,
        };
        assert_ne!(cms1.xmr_channel(), cms2.xmr_channel());
    }

    #[test]
    fn cms_settings_roundtrip_json() {
        let cms = CmsSettings {
            address: "https://cms.example.com".into(),
            key: "secret".into(),
            display_id: "xyz".into(),
            display_name: Some("Reception".into()),
            proxy: None,
        };
        let json = serde_json::to_string_pretty(&cms).unwrap();
        let parsed: CmsSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.address, cms.address);
        assert_eq!(parsed.key, cms.key);
        assert_eq!(parsed.display_id, cms.display_id);
        assert_eq!(parsed.display_name, cms.display_name);
    }
}
