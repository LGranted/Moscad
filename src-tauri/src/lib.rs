use tauri_plugin_shell::ShellExt;

pub mod android_stubs;
pub mod utils;
pub mod commands;
#[cfg(target_os = "android")] pub mod modules;
#[cfg(target_os = "android")] pub use android_stubs::*;
#[cfg(target_os = "android")]
pub use commands::proxy_android_stub::{
    handle_android_stealth_request,
    handle_android_stealth_request_stream,
};

#[tauri::command]
fn show_main_window() {}

#[tauri::command]
fn greet(name: &str) -> String { format!("Hello, {}!", name) }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{info}");
        let _ = std::fs::write("/sdcard/Download/panic.log", &msg);
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            show_main_window,
            #[cfg(target_os = "android")] get_device_profiles,
            #[cfg(target_os = "android")] bind_device_profile,
            #[cfg(target_os = "android")] bind_device_profile_with_profile,
            #[cfg(target_os = "android")] preview_generate_profile,
            #[cfg(target_os = "android")] apply_device_profile,
            #[cfg(target_os = "android")] start_proxy_foreground_service,
            #[cfg(target_os = "android")] start_proxy_service,
            #[cfg(target_os = "android")] stop_proxy_service,
            #[cfg(target_os = "android")] get_proxy_status,
            #[cfg(target_os = "android")] handle_android_stealth_request,
            #[cfg(target_os = "android")] list_accounts,
            #[cfg(target_os = "android")] get_current_account,
            #[cfg(target_os = "android")] add_account,
            #[cfg(target_os = "android")] delete_account,
            #[cfg(target_os = "android")] delete_accounts,
            #[cfg(target_os = "android")] switch_account,
            #[cfg(target_os = "android")] fetch_account_quota,
            #[cfg(target_os = "android")] refresh_account_quota,
            #[cfg(target_os = "android")] refresh_all_quotas,
            #[cfg(target_os = "android")] reorder_accounts,
            #[cfg(target_os = "android")] toggle_proxy_status,
            #[cfg(target_os = "android")] warm_up_all_accounts,
            #[cfg(target_os = "android")] warm_up_account,
            #[cfg(target_os = "android")] start_oauth_login,
            #[cfg(target_os = "android")] complete_oauth_login,
            #[cfg(target_os = "android")] cancel_oauth_login,
            #[cfg(target_os = "android")] prepare_oauth_url,
            #[cfg(target_os = "android")] submit_oauth_code,
            #[cfg(target_os = "android")] import_v1_accounts,
            #[cfg(target_os = "android")] import_from_db,
            #[cfg(target_os = "android")] import_custom_db,
            #[cfg(target_os = "android")] sync_account_from_db,
            #[cfg(target_os = "android")] list_device_versions,
            #[cfg(target_os = "android")] restore_device_version,
            #[cfg(target_os = "android")] delete_device_version,
            #[cfg(target_os = "android")] open_device_folder,
            #[cfg(target_os = "android")] restore_original_device,
            #[cfg(target_os = "android")] update_model_mapping,
            #[cfg(target_os = "android")] generate_api_key,
            #[cfg(target_os = "android")] clear_proxy_session_bindings,
            #[cfg(target_os = "android")] clear_proxy_rate_limit,
            #[cfg(target_os = "android")] clear_all_proxy_rate_limits,
            #[cfg(target_os = "android")] check_proxy_health,
            #[cfg(target_os = "android")] get_preferred_account,
            #[cfg(target_os = "android")] set_preferred_account,
            #[cfg(target_os = "android")] fetch_zai_models,
            #[cfg(target_os = "android")] load_config,
            #[cfg(target_os = "android")] save_config,
            #[cfg(target_os = "android")] get_proxy_stats,
            #[cfg(target_os = "android")] set_proxy_monitor_enabled,
            #[cfg(target_os = "android")] get_proxy_logs_filtered,
            #[cfg(target_os = "android")] get_proxy_logs_count_filtered,
            #[cfg(target_os = "android")] clear_proxy_logs,
            #[cfg(target_os = "android")] get_proxy_log_detail,
            #[cfg(target_os = "android")] enable_debug_console,
            #[cfg(target_os = "android")] disable_debug_console,
            #[cfg(target_os = "android")] is_debug_console_enabled,
            #[cfg(target_os = "android")] get_debug_console_logs,
            #[cfg(target_os = "android")] clear_debug_console_logs,
            #[cfg(target_os = "android")] get_cli_sync_status,
            #[cfg(target_os = "android")] execute_cli_sync,
            #[cfg(target_os = "android")] execute_cli_restore,
            #[cfg(target_os = "android")] get_cli_config_content,
            #[cfg(target_os = "android")] get_opencode_sync_status,
            #[cfg(target_os = "android")] execute_opencode_sync,
            #[cfg(target_os = "android")] execute_opencode_restore,
            #[cfg(target_os = "android")] execute_opencode_clear,
            #[cfg(target_os = "android")] get_opencode_config_content,
            #[cfg(target_os = "android")] get_token_stats_hourly,
            #[cfg(target_os = "android")] get_token_stats_daily,
            #[cfg(target_os = "android")] get_token_stats_weekly,
            #[cfg(target_os = "android")] get_token_stats_by_account,
            #[cfg(target_os = "android")] get_token_stats_summary,
            #[cfg(target_os = "android")] get_token_stats_by_model,
            #[cfg(target_os = "android")] get_token_stats_model_trend_hourly,
            #[cfg(target_os = "android")] get_token_stats_model_trend_daily,
            #[cfg(target_os = "android")] get_token_stats_account_trend_hourly,
            #[cfg(target_os = "android")] get_token_stats_account_trend_daily,
            #[cfg(target_os = "android")] clear_token_stats,
            #[cfg(target_os = "android")] get_data_dir_path,
            #[cfg(target_os = "android")] get_update_settings,
            #[cfg(target_os = "android")] save_update_settings,
            #[cfg(target_os = "android")] is_auto_launch_enabled,
            #[cfg(target_os = "android")] toggle_auto_launch,
            #[cfg(target_os = "android")] get_http_api_settings,
            #[cfg(target_os = "android")] save_http_api_settings,
            #[cfg(target_os = "android")] get_antigravity_path,
            #[cfg(target_os = "android")] get_antigravity_args,
            #[cfg(target_os = "android")] cloudflared_install,
            #[cfg(target_os = "android")] cloudflared_start,
            #[cfg(target_os = "android")] cloudflared_stop,
            #[cfg(target_os = "android")] cloudflared_get_status,
            #[cfg(target_os = "android")] should_check_updates,
            #[cfg(target_os = "android")] check_for_updates,
            #[cfg(target_os = "android")] update_last_check_time,
            #[cfg(target_os = "android")] get_ip_access_logs,
            #[cfg(target_os = "android")] clear_ip_access_logs,
            #[cfg(target_os = "android")] get_ip_stats,
            #[cfg(target_os = "android")] get_ip_token_stats,
            #[cfg(target_os = "android")] get_ip_blacklist,
            #[cfg(target_os = "android")] add_ip_to_blacklist,
            #[cfg(target_os = "android")] remove_ip_from_blacklist,
            #[cfg(target_os = "android")] clear_ip_blacklist,
            #[cfg(target_os = "android")] check_ip_in_blacklist,
            #[cfg(target_os = "android")] get_ip_whitelist,
            #[cfg(target_os = "android")] add_ip_to_whitelist,
            #[cfg(target_os = "android")] remove_ip_from_whitelist,
            #[cfg(target_os = "android")] clear_ip_whitelist,
            #[cfg(target_os = "android")] check_ip_in_whitelist,
            #[cfg(target_os = "android")] get_security_config,
            #[cfg(target_os = "android")] update_security_config,
            #[cfg(target_os = "android")] list_user_tokens,
            #[cfg(target_os = "android")] get_user_token_summary,
            #[cfg(target_os = "android")] create_user_token,
            #[cfg(target_os = "android")] renew_user_token,
            #[cfg(target_os = "android")] delete_user_token,
            #[cfg(target_os = "android")] update_user_token,
            #[cfg(target_os = "android")] get_proxy_pool_config,
            #[cfg(target_os = "android")] get_all_account_bindings,
            #[cfg(target_os = "android")] bind_account_proxy,
            #[cfg(target_os = "android")] unbind_account_proxy,
            #[cfg(target_os = "android")] get_account_proxy_binding,
            #[cfg(target_os = "android")] get_proxy_scheduling_config,
            #[cfg(target_os = "android")] update_proxy_scheduling_config,
            #[cfg(target_os = "android")] open_data_folder,
            #[cfg(target_os = "android")] clear_antigravity_cache,
            #[cfg(target_os = "android")] get_antigravity_cache_paths,
            #[cfg(target_os = "android")] clear_log_cache,
            #[cfg(target_os = "android")] update_account_label,
            #[cfg(target_os = "android")] export_accounts,
            #[cfg(target_os = "android")] reload_proxy_accounts,
            #[cfg(target_os = "android")] get_proxy_logs_paginated,
            #[cfg(target_os = "android")] get_proxy_logs_count,
            #[cfg(target_os = "android")] export_proxy_logs,
            #[cfg(target_os = "android")] export_proxy_logs_json,
            #[cfg(target_os = "android")] get_proxy_logs,
            #[cfg(target_os = "android")] handle_android_stealth_request_stream,
            #[cfg(target_os = "android")] save_text_file,
            #[cfg(target_os = "android")] set_window_theme
        ])
        .setup(|app| {
            #[cfg(target_os = "android")]
            {
                let handle = app.handle().clone();
                let Ok(sidecar_command) = handle.shell().sidecar("sidecar") else { return Ok(()); };
                let _ = sidecar_command.spawn().ok();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
