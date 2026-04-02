use tauri::Manager;
use tauri_plugin_shell::ShellExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle();
            // Запуск нашего Chrome 133 Sidecar
            let sidecar_command = handle.shell().sidecar("sidecar").unwrap();
            let _ = sidecar_command.spawn().ok();
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
