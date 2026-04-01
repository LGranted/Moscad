use tauri::Manager;
use tauri_plugin_shell::process::Command;
use tauri_plugin_shell::ShellExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let shell = app.shell();
            // Запуск Go-сайдкара (Chrome 133 Engine)
            let (mut _rx, _child) = shell
                .sidecar("sidecar")
                .expect("failed to setup sidecar")
                .spawn()
                .expect("failed to spawn sidecar");
            
            println!("Moscad Sidecar (Chrome 133) started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
