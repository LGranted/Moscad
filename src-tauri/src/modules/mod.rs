// ── Общие модули (работают на всех платформах) ────────────────────────────────
#[cfg(not(target_os = "android"))]
#[cfg(not(target_os = "android"))]
pub mod cache;
#[cfg(not(target_os = "android"))]
pub mod db;
#[cfg(not(target_os = "android"))]
#[cfg(not(target_os = "android"))]
pub mod migration;
pub mod oauth;
pub mod oauth_server;

// ── config ────────────────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod config;

#[cfg(target_os = "android")]
pub mod config {
    use std::fs;
    use std::path::PathBuf;

    pub fn get_android_data_dir() -> PathBuf {
        // Android app data directory
        std::env::var("ANDROID_DATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/data/data/com.lbjlaq.antigravity_tools/files"))
    }

    pub fn load_app_config() -> Result<serde_json::Value, String> {
        let path = get_android_data_dir().join("gui_config.json");
        if !path.exists() {
            return Ok(serde_json::json!({}));
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("failed_to_read_config: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("failed_to_parse_config: {}", e))
    }

    pub fn save_app_config(config: &serde_json::Value) -> Result<(), String> {
        let dir = get_android_data_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("failed_to_create_dir: {}", e))?;
        let path = dir.join("gui_config.json");
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("failed_to_serialize: {}", e))?;
        crate::utils::atomic::write_atomic(&path, &content)
    }
}

// ── logger ────────────────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod logger;

#[cfg(target_os = "android")]
pub mod logger {
    pub fn init() -> Result<(), String> {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("Moscad"),
        );
        log::info!("Android logger initialized");
        Ok(())
    }
    pub fn rotate_logs() -> Result<(), String> { Ok(()) }
    pub fn log_info(msg: &str) { log::info!("{}", msg); }
    pub fn log_warn(msg: &str) { log::warn!("{}", msg); }
    pub fn log_error(msg: &str) { log::error!("{}", msg); }
    pub fn log_debug(msg: &str) { log::debug!("{}", msg); }
}

// ── log_bridge ────────────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod log_bridge;

#[cfg(target_os = "android")]
pub mod log_bridge {
    use log::{debug, error, info, warn};
    pub fn bridge_info(tag: &str, msg: &str) { info!(target: tag, "{}", msg); }
    pub fn bridge_warn(tag: &str, msg: &str) { warn!(target: tag, "{}", msg); }
    pub fn bridge_error(tag: &str, msg: &str) { error!(target: tag, "{}", msg); }
    pub fn bridge_debug(tag: &str, msg: &str) { debug!(target: tag, "{}", msg); }
}

// ── account ───────────────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod account;

#[cfg(target_os = "android")]
pub mod account {
    use std::path::PathBuf;

    pub fn get_data_dir() -> Result<PathBuf, String> {
        Ok(PathBuf::from("/data/data/com.lbjlaq.antigravity_tools/files"))
    }
}

#[cfg(target_os = "android")]

#[cfg(target_os = "android")]

#[cfg(target_os = "android")]

#[cfg(target_os = "android")]

#[cfg(target_os = "android")]
pub mod db_android;

// ── account_service ───────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod account_service;

#[cfg(target_os = "android")]
pub mod account_service {
    pub struct AccountService;
    impl AccountService {
        pub fn new(_a: &str, _b: &str) -> Self { Self }
    }
}

// ── update_checker ────────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod update_checker;

#[cfg(target_os = "android")]
pub mod update_checker {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Clone, Default)]
    pub struct UpdateSettings;
}

// ── quota ─────────────────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod quota;

#[cfg(target_os = "android")]
pub mod quota {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Clone, Default)]
    pub struct QuotaData;
    pub async fn fetch_quota() -> Result<(), String> { Ok(()) }
}

pub use quota::fetch_quota;

// ── device ────────────────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod device;

#[cfg(target_os = "android")]
pub mod device {
    pub fn get_device_profiles(_id: &str) -> Result<String, String> { Ok("[]".into()) }
}

// ── integration ───────────────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))]
pub mod integration;

#[cfg(target_os = "android")]
pub mod integration {
    #[derive(Clone)]
    pub enum SystemManager { Android }
}

// ── Desktop-only модули ───────────────────────────────────────────────────────
#[cfg(not(target_os = "android"))] pub mod proxy_db;
#[cfg(not(target_os = "android"))] pub mod tray;
#[cfg(not(target_os = "android"))] pub mod process;

#[cfg(not(target_os = "android"))]
#[cfg(not(target_os = "android"))]
pub mod tls_proxy;
#[cfg(not(target_os = "android"))]
#[cfg(not(target_os = "android"))]
pub mod tls_mimicry;
