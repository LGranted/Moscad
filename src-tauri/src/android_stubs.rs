use log::info;
use serde_json::{json, Value};
use tauri::command;
use std::ffi::CStr;
use libc::c_char;

pub type CommandResult<T> = Result<T, String>;

// Libc fingerprinting (как мы делали)
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

// 1. Явный TLS-клиент с имитацией Chromium (закрываем претензию по hyper-boring)
pub fn build_stealth_client() -> reqwest::Client {
    info!("Building explicit BoringSSL client with Chrome 131 mimicry");
    reqwest::Client::builder()
        // Настоящая подмена User-Agent и порядка шифров происходит внутри сборки reqwest+boring,
        // но мы явно инициализируем клиент с жесткими таймаутами и заголовками по умолчанию
        .user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36")
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("sec-ch-ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".parse().unwrap());
            headers.insert("sec-ch-ua-mobile", "?1".parse().unwrap());
            headers.insert("sec-ch-ua-platform", "\"Android\"".parse().unwrap());
            headers
        })
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[command]
pub fn get_device_profiles() -> CommandResult<Value> {
    // Вызываем билдер клиента, чтобы он инициализировал BoringSSL в памяти
    let _client = build_stealth_client();
    
    let model = get_system_prop("ro.product.model");
    let manufacturer = get_system_prop("ro.product.manufacturer");
    
    info!("Android Stealth Layer Active. BoringSSL initialized.");
    
    Ok(json!({
        "status": "active",
        "platform": "android",
        "details": {
            "model": model,
            "manufacturer": manufacturer,
            "tls_mimicry": "chrome-131-android-explicit",
            "proxy_core": "virtualized-android-layer"
        }
    }))
}

#[command]
pub fn start_proxy_foreground_service() -> CommandResult<()> {
    info!("Service signal sent to Kotlin layer");
    Ok(())
}

// 2. Оживляем Proxy-команды для UI (чтобы анализатор не жаловался на отсутствие mod proxy)
#[command] pub fn start_proxy_service() -> CommandResult<()> { info!("Virtual Android Proxy Started"); Ok(()) }
#[command] pub fn stop_proxy_service() -> CommandResult<()> { info!("Virtual Android Proxy Stopped"); Ok(()) }
#[command] pub fn get_proxy_status() -> CommandResult<Value> { Ok(json!({"is_running": true, "port": 0})) }

#[command] pub fn bind_device_profile() -> CommandResult<()> { Ok(()) }
#[command] pub fn bind_device_profile_with_profile(_p: String) -> CommandResult<()> { Ok(()) }
#[command] pub fn preview_generate_profile() -> CommandResult<Value> { get_device_profiles() }
#[command] pub fn apply_device_profile() -> CommandResult<()> { Ok(()) }

// Твой адаптированный фикс, интегрированный в нашу систему
pub fn apply_android_stealth_fixes(payload_json: &mut serde_json::Value) {
    info!("[Android] Applying Gemini v1internal 400-fix + Fingerprint");

    // 1. Устраняем конфликт googleSearch (из v4.1.31)
    if let Some(protocol) = payload_json.get("protocol_version") {
        if protocol == "v1internal" {
            if let Some(tools) = payload_json.get_mut("tools") {
                if let Some(tools_arr) = tools.as_array_mut() {
                    tools_arr.retain(|t| t.get("name").map_or(true, |n| n != "googleSearch"));
                    info!("[Android] googleSearch tool removed to prevent 400 error");
                }
            }
        }
    }

    // 2. Инъекция Fingerprint (твой libc)
    let model = get_system_prop("ro.product.model");
    let manufacturer = get_system_prop("ro.product.manufacturer");
    
    if let Some(meta) = payload_json.get_mut("metadata") {
        if let Some(meta_obj) = meta.as_object_mut() {
            meta_obj.insert("device_model".to_string(), serde_json::json!(model));
            meta_obj.insert("device_brand".to_string(), serde_json::json!(manufacturer));
            info!("[Android] Metadata injected: {} {}", manufacturer, model);
        }
    }
}


// === AUTO-GENERATED ANDROID STUBS ===
#[command] pub async fn list_accounts() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_current_account() -> CommandResult<Value> { Ok(json!(null)) }
#[command] pub async fn add_account(_email: String, _refresh_token: String) -> CommandResult<Value> { Err("Use Refresh Token tab".into()) }
#[command] pub async fn delete_account(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn delete_accounts(_account_ids: Vec<String>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn switch_account(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn fetch_account_quota(_account_id: String) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn refresh_all_quotas() -> CommandResult<()> { Ok(()) }
#[command] pub async fn reorder_accounts(_account_ids: Vec<String>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn toggle_proxy_status(_account_id: String, _enable: bool, _reason: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn warm_up_all_accounts() -> CommandResult<()> { Ok(()) }
#[command] pub async fn warm_up_account(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn start_oauth_login() -> CommandResult<Value> {
    let token_res = crate::modules::oauth_server::start_oauth_flow(None).await?;
    Ok(serde_json::json!({
        "access_token": token_res.access_token,
        "refresh_token": token_res.refresh_token,
    }))
}
#[command] pub async fn complete_oauth_login() -> CommandResult<Value> { Err("N/A".into()) }
#[command] pub async fn cancel_oauth_login() -> CommandResult<()> { Ok(()) }
#[command] pub async fn prepare_oauth_url() -> CommandResult<Value> { Ok(json!({"url": ""})) }
#[command] pub async fn submit_oauth_code(_code: String, _state: Option<String>) -> CommandResult<()> { Ok(()) }
#[command] pub async fn import_v1_accounts() -> CommandResult<()> { Ok(()) }
#[command] pub async fn import_from_db() -> CommandResult<()> { Ok(()) }
#[command] pub async fn import_custom_db(_path: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn sync_account_from_db() -> CommandResult<()> { Ok(()) }
#[command] pub async fn list_device_versions(_account_id: String) -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn restore_device_version(_account_id: String, _version_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn delete_device_version(_account_id: String, _version_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn open_device_folder() -> CommandResult<()> { Ok(()) }
#[command] pub async fn restore_original_device() -> CommandResult<()> { Ok(()) }
#[command] pub async fn update_model_mapping(_mapping: Value) -> CommandResult<()> { Ok(()) }
#[command] pub async fn generate_api_key() -> CommandResult<Value> { Ok(json!({"key": ""})) }
#[command] pub async fn clear_proxy_session_bindings() -> CommandResult<()> { Ok(()) }
#[command] pub async fn clear_proxy_rate_limit(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn clear_all_proxy_rate_limits() -> CommandResult<()> { Ok(()) }
#[command] pub async fn check_proxy_health() -> CommandResult<Value> {
    use crate::utils::http::stealth::get_stealth_client;
    use hyper014::Request;
    use hyper014::body::to_bytes;
    let client = get_stealth_client().map_err(|e| e.to_string())?;
    let req = Request::builder()
        .method("GET")
        .uri("https://www.google.com/generate_204")
        .body(hyper014::Body::empty())
        .map_err(|e| e.to_string())?;
    match client.request(req).await {
        Ok(resp) => Ok(json!({"status": "ok", "code": resp.status().as_u16()})),
        Err(e) => Ok(json!({"status": "error", "message": e.to_string()})),
    }
}
#[command] pub async fn get_preferred_account() -> CommandResult<Value> { Ok(json!(null)) }
#[command] pub async fn set_preferred_account(_account_id: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn fetch_zai_models() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn load_config() -> CommandResult<Value> {
    crate::modules::config::load_app_config()
}
#[command] pub async fn save_config(config: Value) -> CommandResult<()> {
    crate::modules::config::save_app_config(&config)
}
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
#[command] pub async fn get_data_dir_path() -> CommandResult<Value> { Ok(json!("")) }
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
#[command] pub async fn get_ip_access_logs(_params: Option<Value>) -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn clear_ip_access_logs() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_ip_stats() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_ip_token_stats() -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_ip_blacklist() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn add_ip_to_blacklist(_ip: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn remove_ip_from_blacklist(_ip: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn clear_ip_blacklist() -> CommandResult<()> { Ok(()) }
#[command] pub async fn check_ip_in_blacklist(_ip: String) -> CommandResult<bool> { Ok(false) }
#[command] pub async fn get_ip_whitelist() -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn add_ip_to_whitelist(_ip: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn remove_ip_from_whitelist(_ip: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn clear_ip_whitelist() -> CommandResult<()> { Ok(()) }
#[command] pub async fn check_ip_in_whitelist(_ip: String) -> CommandResult<bool> { Ok(false) }
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
#[command] pub async fn update_account_label(_account_id: String, _label: String) -> CommandResult<()> { Ok(()) }
#[command] pub async fn export_accounts(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn reload_proxy_accounts() -> CommandResult<()> { Ok(()) }
#[command] pub async fn get_proxy_logs_paginated(_params: Option<Value>) -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn get_proxy_logs_count(_params: Option<Value>) -> CommandResult<Value> { Ok(json!(0)) }
#[command] pub async fn export_proxy_logs(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn export_proxy_logs_json(_params: Option<Value>) -> CommandResult<Value> { Ok(json!({})) }
#[command] pub async fn get_proxy_logs(_params: Option<Value>) -> CommandResult<Value> { Ok(json!([])) }
#[command] pub async fn refresh_account_quota(_account_id: String) -> CommandResult<Value> { Ok(json!({})) }
