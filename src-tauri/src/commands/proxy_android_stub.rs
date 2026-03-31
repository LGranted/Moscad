use tauri::Emitter;
use crate::utils::http::stealth::get_stealth_client;
use hyper014::{Request, Body};
use hyper014::body::to_bytes;
use std::collections::HashMap;

use crate::utils::fingerprint::FingerprintConfig;

fn get_chrome_ua() -> String {
    crate::utils::fingerprint::FingerprintConfig::current().user_agent.clone()
}

fn get_antigravity_version() -> String {
    crate::utils::fingerprint::FingerprintConfig::current().antigravity_version.clone()
}

fn get_goog_api_client() -> String {
    "gl-node/22.18.0".to_string()
}

#[derive(Clone)]
pub struct ProxyServiceState;

impl ProxyServiceState {
    pub fn new() -> Self { Self }
}

#[tauri::command]
pub async fn handle_android_stealth_request(
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let client = get_stealth_client().map_err(|e| e.to_string())?;

    let body = if body.is_empty() {
        Body::empty()
    } else {
        Body::from(body)
    };

    let mut builder = Request::builder()
        .method(method.as_str())
        .uri(&url);

    // Memory Token Injection — подставляем свежий токен если есть
    let injected_token = crate::utils::token_store::get_fresh_token().await;

    // Пробрасываем заголовки от фронтенда
    for (k, v) in &headers {
        let key_lower = k.to_lowercase();
        // Пропускаем заголовки которые перезапишем ниже
        if key_lower == "user-agent" || key_lower == "sec-ch-ua" || key_lower == "sec-ch-ua-mobile" || key_lower == "sec-ch-ua-platform" {
            continue;
        }
        // Заменяем Authorization свежим токеном из store
        if key_lower == "authorization" {
            if let Some(ref token) = injected_token {
                builder = builder.header("authorization", format!("Bearer {}", token));
                continue;
            }
        }
        builder = builder.header(k, v);
    }

    // Принудительно синхронизируем UA с TLS fingerprint
    // Десктопный Antigravity fingerprint
    builder = builder.header("user-agent", get_chrome_ua());
    builder = builder.header("x-goog-api-client", get_goog_api_client());
    builder = builder.header("x-client-name", "antigravity");
    builder = builder.header("x-client-version", get_antigravity_version());
    // Уникальный android_id для каждого аккаунта — изолируем аккаунты
    if let Some(android_id) = crate::utils::token_store::get_current_android_id() {
        builder = builder.header("x-client-device-id", android_id);
    }

    let request = builder
        .body(body)
        .map_err(|e| e.to_string())?;

    let response = client.request(request).await.map_err(|e| e.to_string())?;
    let bytes = to_bytes(response.into_body()).await.map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

#[tauri::command]
pub async fn handle_android_stealth_request_stream(
    app: tauri::AppHandle,
    url: String,
    method: String,
    headers: std::collections::HashMap<String, String>,
    body: Vec<u8>,
    event_id: String,
) -> Result<(), String> {
    use hyper014::body::HttpBody;

    let client = get_stealth_client().map_err(|e| e.to_string())?;

    let body_data = if body.is_empty() {
        hyper014::Body::empty()
    } else {
        hyper014::Body::from(body)
    };

    let mut builder = hyper014::Request::builder()
        .method(method.as_str())
        .uri(&url);

    // Memory Token Injection для стриминга
    let injected_token = crate::utils::token_store::get_fresh_token().await;

    for (k, v) in &headers {
        let key_lower = k.to_lowercase();
        if key_lower == "user-agent" || key_lower == "sec-ch-ua" || key_lower == "sec-ch-ua-mobile" || key_lower == "sec-ch-ua-platform" {
            continue;
        }
        if key_lower == "authorization" {
            if let Some(ref token) = injected_token {
                builder = builder.header("authorization", format!("Bearer {}", token));
                continue;
            }
        }
        builder = builder.header(k, v);
    }

    // Десктопный Antigravity fingerprint
    builder = builder.header("user-agent", get_chrome_ua());
    builder = builder.header("x-goog-api-client", get_goog_api_client());
    builder = builder.header("x-client-name", "antigravity");
    builder = builder.header("x-client-version", get_antigravity_version());
    // Уникальный android_id для каждого аккаунта — изолируем аккаунты
    if let Some(android_id) = crate::utils::token_store::get_current_android_id() {
        builder = builder.header("x-client-device-id", android_id);
    }

    let request = builder
        .body(body_data)
        .map_err(|e| e.to_string())?;

    let response = client.request(request).await.map_err(|e| e.to_string())?;
    let mut body = response.into_body();

    while let Some(chunk) = body.data().await {
        match chunk {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                let _ = app.emit(&format!("stream-chunk-{}", event_id), text);
            }
            Err(e) => {
                let _ = app.emit(&format!("stream-error-{}", event_id), e.to_string());
                break;
            }
        }
    }

    let _ = app.emit(&format!("stream-done-{}", event_id), "");
    Ok(())
}
