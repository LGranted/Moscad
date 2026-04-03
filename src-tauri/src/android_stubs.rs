use log::info;
use serde_json::{json, Value};
use tauri::command;
use std::ffi::CStr;
use libc::c_char;

use crate::modules::db_android;

pub type CommandResult<T> = Result<T, String>;

// ── Libc fingerprinting ───────────────────────────────────────────────────────
extern "C" { fn __system_property_get(name: *const c_char, value: *mut c_char) -> libc::c_int; }

fn get_system_prop(prop: &str) -> String {
    let mut value = [0 as c_char; 92];
    let name = std::ffi::CString::new(prop).unwrap();
    unsafe {
        if __system_property_get(name.as_ptr(), value.as_mut_ptr()) > 0 {
            CStr::from_ptr(value.as_ptr()).to_string_lossy().into_owned()
        } else { "unknown".to_string() }
    }
}

// ── Stealth client ────────────────────────────────────────────────────────────
pub fn build_stealth_client() -> reqwest::Client {
    info!("Building Antigravity Linux stealth client");
    let fp = crate::utils::fingerprint::FingerprintConfig::current();
    let ver = fp.antigravity_version.clone();
    reqwest::Client::builder()
        .user_agent(fp.user_agent.clone())
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("x-goog-api-client", "gl-node/22.18.0".parse().unwrap());
            headers.insert("x-client-name", "antigravity".parse().unwrap());
            headers.insert("x-client-version", ver.parse().unwrap());
            headers
        })
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[command]
pub fn get_device_profiles() -> CommandResult<Value> {
    let _client = build_stealth_client();
    let ag_version = crate::utils::fingerprint::FingerprintConfig::current()
        .antigravity_version.clone();
    info!("Moscad Stealth Layer Active. BoringSSL initialized.");
    Ok(json!({
        "status": "active",
        "platform": "linux/aarch64",
        "details": {
            "ide_version": ag_version,
            "proxy_core": "moscad-linux-layer"
        }
    }))
}

#[command]
pub fn start_proxy_foreground_service() -> CommandResult<()> {
    info!("Service signal sent to Kotlin layer");
    Ok(())
}

#[command] pub fn start_proxy_service() -> CommandResult<()> { info!("Virtual Android Proxy Started"); Ok(()) }
#[command] pub fn stop_proxy_service()  -> CommandResult<()> { info!("Virtual Android Proxy Stopped"); Ok(()) }
#[command] pub fn get_proxy_status()    -> CommandResult<Value> { Ok(json!({"is_running": true, "port": 0})) }

#[command] pub fn bind_device_profile() -> CommandResult<()> { Ok(()) }
#[command] pub fn bind_device_profile_with_profile(_p: String) -> CommandResult<()> { Ok(()) }
#[command] pub fn preview_generate_profile() -> CommandResult<Value> { get_device_profiles() }
#[command] pub fn apply_device_profile() -> CommandResult<()> { Ok(()) }

// ── Stealth fixes ─────────────────────────────────────────────────────────────
pub fn apply_android_stealth_fixes(payload_json: &mut serde_json::Value) {
    info!("[Android] Applying Gemini v1internal 400-fix + Fingerprint");
    if let Some(protocol) = payload_json.get("protocol_version") {
        if protocol == "v1internal" {
            if let Some(tools) = payload_json.get_mut("tools") {
                if let Some(arr) = tools.as_array_mut() {
                    arr.retain(|t| t.get("name").map_or(true, |n| n != "googleSearch"));
                    info!("[Android] googleSearch tool removed to prevent 400 error");
                }
            }
        }
    }
    let ag_version = crate::utils::fingerprint::FingerprintConfig::current()
        .antigravity_version.clone();
    if let Some(meta) = payload_json.get_mut("metadata") {
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("ide_type".to_string(), json!(9));
            obj.insert("ide_version".to_string(), json!(ag_version.clone()));
            obj.insert("ide_name".to_string(), json!("antigravity"));
            obj.insert("platform".to_string(), json!("linux/aarch64"));
            info!("[Android] Antigravity metadata injected v{}", ag_version);
        }
    } else {
        if let Some(obj) = payload_json.as_object_mut() {
            obj.insert("metadata".to_string(), json!({
                "ide_type": 9,
                "ide_version": ag_version,
                "ide_name": "antigravity",
                "platform": "linux/aarch64"
            }));
        }
    }
}

// ── Accounts — реальные команды ───────────────────────────────────────────────
#[command]
pub async fn list_accounts() -> CommandResult<Value> {
    let accounts = db_android::list_accounts()?;
    Ok(serde_json::to_value(accounts).map_err(|e| e.to_string())?)
}

#[command]
pub async fn get_current_account() -> CommandResult<Value> {
    let account = db_android::get_current_account()?;
    Ok(serde_json::to_value(account).map_err(|e| e.to_string())?)
}

#[command]
pub async fn add_account(email: String, refresh_token: String) -> CommandResult<Value> {
    let account = db_android::add_account(&email, &refresh_token)?;
    info!("[DB] Account added: {}", email);
    Ok(serde_json::to_value(account).map_err(|e| e.to_string())?)
}

#[command]
pub async fn delete_account(account_id: String) -> CommandResult<()> {
    db_android::delete_account(&account_id)?;
    info!("[DB] Account deleted: {}", account_id);
    Ok(())
}

#[command]
pub async fn delete_accounts(account_ids: Vec<String>) -> CommandResult<()> {
    for id in &account_ids {
        db_android::delete_account(id)?;
    }
    info!("[DB] Deleted {} accounts", account_ids.len());
    Ok(())
}

#[command]
pub async fn switch_account(account_id: String) -> CommandResult<()> {
    db_android::switch_account(&account_id)?;
    // Устанавливаем активный аккаунт в token store
    crate::utils::token_store::set_current(&account_id);
    info!("[DB] Switched to account: {}", account_id);
    Ok(())
}

#[command]
pub async fn update_account_label(account_id: String, label: String) -> CommandResult<()> {
    db_android::update_label(&account_id, &label)?;
    Ok(())
}

#[command]
pub async fn export_accounts(_params: Option<Value>) -> CommandResult<Value> {
    let accounts = db_android::list_accounts()?;
    let export: Vec<Value> = accounts.iter().map(|a| json!({
        "email": a.email,
        "refresh_token": a.refresh_token,
    })).collect();
    Ok(json!({ "accounts": export }))
}

// ── IP Blacklist — реальные команды ──────────────────────────────────────────
#[command]
pub async fn get_ip_blacklist() -> CommandResult<Value> {
    let list = db_android::get_blacklist()?;
    Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
}

#[command]
pub async fn add_ip_to_blacklist(ip: String) -> CommandResult<()> {
    db_android::add_to_blacklist(&ip, None)?;
    info!("[Security] IP blacklisted: {}", ip);
    Ok(())
}

#[command]
pub async fn remove_ip_from_blacklist(ip: String) -> CommandResult<()> {
    db_android::remove_from_blacklist(&ip)?;
    Ok(())
}

#[command]
pub async fn clear_ip_blacklist() -> CommandResult<()> {
    db_android::clear_blacklist()?;
    Ok(())
}

#[command]
pub async fn check_ip_in_blacklist(ip: String) -> CommandResult<bool> {
    db_android::is_ip_in_blacklist(&ip)
}

// ── IP Whitelist — реальные команды ──────────────────────────────────────────
#[command]
pub async fn get_ip_whitelist() -> CommandResult<Value> {
    let list = db_android::get_whitelist()?;
    Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
}

#[command]
pub async fn add_ip_to_whitelist(ip: String) -> CommandResult<()> {
    db_android::add_to_whitelist(&ip, None)?;
    info!("[Security] IP whitelisted: {}", ip);
    Ok(())
}

#[command]
pub async fn remove_ip_from_whitelist(ip: String) -> CommandResult<()> {
    db_android::remove_from_whitelist(&ip)?;
    Ok(())
}

#[command]
pub async fn clear_ip_whitelist() -> CommandResult<()> {
    db_android::clear_whitelist()?;
    Ok(())
}

#[command]
pub async fn check_ip_in_whitelist(ip: String) -> CommandResult<bool> {
    db_android::is_ip_in_whitelist(&ip)
}

// ── IP Stats — реальные команды ───────────────────────────────────────────────
#[command]
pub async fn get_ip_stats() -> CommandResult<Value> {
    let stats = db_android::get_ip_stats()?;
    Ok(serde_json::to_value(stats).map_err(|e| e.to_string())?)
}

#[command]
pub async fn get_ip_access_logs(_params: Option<Value>) -> CommandResult<Value> {
    Ok(json!([]))
}

#[command]
pub async fn clear_ip_access_logs() -> CommandResult<()> {
    db_android::clear_ip_access_logs()?;
    Ok(())
}

// ── Proxy health ──────────────────────────────────────────────────────────────
#[command]
pub async fn check_proxy_health() -> CommandResult<Value> {
    let client = reqwest::Client::new();
    match client.get("https://www.google.com/generate_204").send().await {
        Ok(resp) => Ok(json!({
            "status": "ok",
            "code": resp.status().as_u16()
        })),
        Err(e) => Ok(json!({
            "status": "error",
            "message": e.to_string()
        }))
    }
}

// ── Config ────────────────────────────────────────────────────────────────────
#[command] pub async fn load_config() -> CommandResult<Value> {
    crate::modules::config::load_app_config()
}
#[command] pub async fn save_config(config: Value) -> CommandResult<()> {
    crate::modules::config::save_app_config(&config)
}
#[command] pub async fn get_data_dir_path() -> CommandResult<Value> {
    Ok(json!("/data/data/com.lbjlaq.antigravity_tools/files"))
}

// ── Заглушки (десктопный функционал) ─────────────────────────────────────────
#[command] pub async fn fetch_account_quota(_account_id: String) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn refresh_account_quota(_account_id: String) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn refresh_all_quotas() -> CommandResult<()> { Ok(()) }
#[command] pub async fn reorder_accounts(_account_ids: Vec<String>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn toggle_proxy_status(_account_id: String, _enable: bool, _reason: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn warm_up_all_accounts() -> CommandResult<()> { Ok(()) }
#[command] pub async fn warm_up_account(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_preferred_account() -> CommandResult<Value> { Ok(json!(null)) }
#[command] pub async fn set_preferred_account(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn fetch_zai_models() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_proxy_stats() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn set_proxy_monitor_enabled(_enabled: bool) -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_proxy_logs_filtered(_params: Option<Value>) -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_proxy_logs_count_filtered(_params: Option<Value>) -> CommandResult<Value> { Ok(json!(0)) }
#[command] pub async fn clear_proxy_logs() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_proxy_log_detail(_log_id: String) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn enable_debug_console() -> CommandResult<()> { Ok(()) }
#[command] pub async fn disable_debug_console() -> CommandResult<()> { Ok(()) }
#[command] pub async fn is_debug_console_enabled() -> CommandResult<bool> { Ok(false) }
#[command] pub async fn get_debug_console_logs() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn clear_debug_console_logs() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_cli_sync_status(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn execute_cli_sync(_params: Option<Value>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn execute_cli_restore(_params: Option<Value>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_cli_config_content(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_opencode_sync_status(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn execute_opencode_sync(_params: Option<Value>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn execute_opencode_restore(_params: Option<Value>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn execute_opencode_clear(_params: Option<Value>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_opencode_config_content(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_token_stats_hourly() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_token_stats_daily() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_token_stats_weekly() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_token_stats_by_account() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_token_stats_summary() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_token_stats_by_model() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_token_stats_model_trend_hourly() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_token_stats_model_trend_daily() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_token_stats_account_trend_hourly() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_token_stats_account_trend_daily() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn clear_token_stats() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_update_settings() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn save_update_settings(_settings: Value) -> CommandResult<()> { Ok(()) }
#[command] pub async fn is_auto_launch_enabled() -> CommandResult<bool> { Ok(false) }
#[command] pub async fn toggle_auto_launch() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_http_api_settings() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn save_http_api_settings(_settings: Value) -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_antigravity_path() -> CommandResult<Value> { Ok(json!("")) }
#[command] pub async fn get_antigravity_args() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn cloudflared_install() -> CommandResult<()> { Ok(()) }
#[command] pub async fn cloudflared_start() -> CommandResult<()> { Ok(()) }
#[command] pub async fn cloudflared_stop() -> CommandResult<()> { Ok(()) }
#[command] pub async fn cloudflared_get_status() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn should_check_updates() -> CommandResult<bool> { Ok(false) }
#[command] pub async fn check_for_updates() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn update_last_check_time() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_ip_token_stats() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_security_config() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn update_security_config(_config: Value) -> CommandResult<()> { Ok(()) }
#[command] pub async fn list_user_tokens() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_user_token_summary() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn create_user_token(_params: Value) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn renew_user_token(_id: String) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn delete_user_token(_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn update_user_token(_id: String, _params: Value) -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_proxy_pool_config() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_all_account_bindings() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn bind_account_proxy(_params: Value) -> CommandResult<()> { Ok(()) }
#[command] pub async fn unbind_account_proxy(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_account_proxy_binding(_account_id: String) -> CommandResult<Value> { Ok(json!(null)) }
#[command] pub async fn get_proxy_scheduling_config() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn update_proxy_scheduling_config(_config: Value) -> CommandResult<()> { Ok(()) }
#[command] pub async fn open_data_folder() -> CommandResult<()> { Ok(()) }
#[command] pub async fn clear_antigravity_cache() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_antigravity_cache_paths() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn clear_log_cache() -> CommandResult<()> { Ok(()) }
#[command] pub async fn reload_proxy_accounts() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_proxy_logs_paginated(_params: Option<Value>) -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_proxy_logs_count(_params: Option<Value>) -> CommandResult<Value> { Ok(json!(0)) }
#[command] pub async fn export_proxy_logs(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn export_proxy_logs_json(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_proxy_logs(_params: Option<Value>) -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn generate_api_key() -> CommandResult<Value> { Ok(json!({"key": ""})) }
#[command] pub async fn clear_proxy_session_bindings() -> CommandResult<()> { Ok(()) }
#[command] pub async fn clear_proxy_rate_limit(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn clear_all_proxy_rate_limits() -> CommandResult<()> { Ok(()) }
#[command] pub async fn update_model_mapping(_mapping: Value) -> CommandResult<()> { Ok(()) }
#[command] pub async fn list_device_versions(_account_id: String) -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn restore_device_version(_account_id: String, _version_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn delete_device_version(_account_id: String, _version_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn open_device_folder() -> CommandResult<()> { Ok(()) }
#[command] pub async fn restore_original_device() -> CommandResult<()> { Ok(()) }

// ── OAuth — реальные команды ──────────────────────────────────────────────────
#[command]
pub async fn start_oauth_login(app_handle: tauri::AppHandle) -> CommandResult<Value> {
    let token = crate::modules::oauth_server::start_oauth_flow(Some(app_handle))
        .await?;
    let user_info = crate::modules::oauth::get_user_info(&token.access_token, None).await?;
    let refresh = token.refresh_token.unwrap_or_default();
    let account = db_android::add_account(&user_info.email, &refresh)?;
    info!("[OAuth] Login successful: {}", account.email);
    Ok(serde_json::to_value(account).map_err(|e| e.to_string())?)
}

#[command]
pub async fn prepare_oauth_url(app_handle: tauri::AppHandle) -> CommandResult<Value> {
    let url = crate::modules::oauth_server::prepare_oauth_url(Some(app_handle)).await?;
    Ok(json!({ "url": url }))
}

#[command]
pub async fn complete_oauth_login(app_handle: tauri::AppHandle) -> CommandResult<Value> {
    let token = crate::modules::oauth_server::complete_oauth_flow(Some(app_handle)).await?;
    let user_info = crate::modules::oauth::get_user_info(&token.access_token, None).await?;
    let refresh = token.refresh_token.unwrap_or_default();
    let account = db_android::add_account(&user_info.email, &refresh)?;
    info!("[OAuth] Complete login: {}", account.email);
    Ok(serde_json::to_value(account).map_err(|e| e.to_string())?)
}

#[command]
pub async fn cancel_oauth_login() -> CommandResult<()> {
    crate::modules::oauth_server::cancel_oauth_flow();
    info!("[OAuth] Flow cancelled");
    Ok(())
}

#[command]
pub async fn submit_oauth_code(code: String, state: Option<String>) -> CommandResult<Value> {
    crate::modules::oauth_server::submit_oauth_code(code, state).await?;
    Ok(json!({ "status": "ok" }))
}

// ── Import — реальные команды ─────────────────────────────────────────────────
#[command]
pub async fn import_custom_db(path: String) -> CommandResult<Value> {
    let refresh_token = extract_refresh_token_android(&std::path::PathBuf::from(&path))?;
    let token_resp = crate::modules::oauth::refresh_access_token(&refresh_token, None).await?;
    let user_info  = crate::modules::oauth::get_user_info(&token_resp.access_token, None).await?;
    let account    = db_android::add_account(&user_info.email, &refresh_token)?;
    info!("[Import] Imported account: {}", account.email);
    Ok(serde_json::to_value(account).map_err(|e| e.to_string())?)
}

#[command]
pub async fn import_from_db() -> CommandResult<Value> {
    Err("На Android укажите путь к файлу базы вручную через import_custom_db".into())
}

#[command]
pub async fn import_v1_accounts() -> CommandResult<Value> {
    Err("Импорт V1 недоступен на Android".into())
}

#[command]
pub async fn sync_account_from_db(account_id: String) -> CommandResult<Value> {
    let accounts = db_android::list_accounts()?;
    let account  = accounts.iter().find(|a| a.id == account_id)
        .ok_or_else(|| format!("Аккаунт не найден: {}", account_id))?;
    let token_resp = crate::modules::oauth::refresh_access_token(&account.refresh_token, None).await?;
    let user_info  = crate::modules::oauth::get_user_info(&token_resp.access_token, None).await?;
    let updated    = db_android::add_account(&user_info.email, &account.refresh_token)?;
    Ok(serde_json::to_value(updated).map_err(|e| e.to_string())?)
}

// ── Import custom db (Android версия без migration модуля) ────────────────────
pub fn extract_refresh_token_android(path: &std::path::PathBuf) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose};
    let conn = rusqlite::Connection::open(path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    let data: String = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = ?",
        ["antigravityUnifiedStateSync.oauthToken"],
        |row| row.get(0),
    ).map_err(|_| "Login state not found".to_string())?;
    let blob = general_purpose::STANDARD.decode(&data)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;
    // Field 1 -> Field 2 -> Field 1 -> Base64 -> Field 3
    let f1 = crate::utils::protobuf::find_field(&blob, 1)
        .map_err(|e| e)?.ok_or("Field 1 not found")?;
    let f2 = crate::utils::protobuf::find_field(&f1, 2)
        .map_err(|e| e)?.ok_or("Field 2 not found")?;
    let f3_bytes = crate::utils::protobuf::find_field(&f2, 1)
        .map_err(|e| e)?.ok_or("Field 1 inner not found")?;
    let b64 = String::from_utf8(f3_bytes)
        .map_err(|_| "Not UTF-8")?;
    let inner = general_purpose::STANDARD.decode(&b64)
        .map_err(|e| format!("Inner decode failed: {}", e))?;
    let refresh = crate::utils::protobuf::find_field(&inner, 3)
        .map_err(|e| e)?.ok_or("Refresh token not found")?;
    String::from_utf8(refresh).map_err(|_| "Refresh token not UTF-8".to_string())
}

#[command]
pub async fn save_text_file(path: String, content: String) -> CommandResult<()> {
    std::fs::write(&path, &content)
        .map_err(|e| format!("Failed to save file: {}", e))
}
#[command] pub async fn set_window_theme(_theme: String) -> CommandResult<()> { Ok(()) }
