pub struct CloudflaredState;
impl CloudflaredState {
    pub fn new() -> Self { Self }
    pub fn inner(&self) -> &Self { self }
}
impl Clone for CloudflaredState {
    fn clone(&self) -> Self { Self }
}
#[tauri::command] pub async fn cloudflared_check() -> Result<(), String> { Err("N/A".into()) }
#[tauri::command] pub async fn cloudflared_install() -> Result<(), String> { Err("N/A".into()) }
#[tauri::command] pub async fn cloudflared_start() -> Result<(), String> { Err("N/A".into()) }
#[tauri::command] pub async fn cloudflared_stop() -> Result<(), String> { Err("N/A".into()) }
#[tauri::command] pub async fn cloudflared_get_status() -> Result<(), String> { Err("N/A".into()) }
