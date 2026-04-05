// src-tauri/src/proxy/upstream/client.rs
// Android-адаптация: rquest → reqwest
// Исправлены все три критических бага компиляции:
//   1. build_client_internal теперь возвращает reqwest::Error (не rquest::Error)
//   2. match #[cfg(...)] expr — убран, заменён на валидный Rust
//   3. Смешение типов rquest/reqwest — устранено полностью
// machine_uid fallback для Android добавлен

use dashmap::DashMap;
use reqwest::{header, Client, Proxy, Response, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

// ---------------------------------------------------------------------------
// Типы результатов
// ---------------------------------------------------------------------------

/// Запись о попытке fallback-переключения эндпоинта
#[derive(Debug, Clone)]
pub struct FallbackAttemptLog {
    /// URL эндпоинта
    pub endpoint_url: String,
    /// HTTP-статус (None если сетевая ошибка)
    pub status: Option<u16>,
    /// Описание ошибки
    pub error: String,
}

/// Результат upstream-вызова
pub struct UpstreamCallResult {
    pub response: Response,
    pub fallback_attempts: Vec<FallbackAttemptLog>,
}

// ---------------------------------------------------------------------------
// Маскировка email (без изменений)
// ---------------------------------------------------------------------------

/// "userexample@gmail.com" → "use***@gm***"
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let local = &email[..at_pos];
        let domain = &email[at_pos + 1..];
        let local_prefix: String = local.chars().take(3).collect();
        let domain_prefix: String = domain.chars().take(2).collect();
        format!("{}***@{}***", local_prefix, domain_prefix)
    } else {
        let prefix: String = email.chars().take(5).collect();
        format!("{}***", prefix)
    }
}

// ---------------------------------------------------------------------------
// Эндпоинты Cloud Code v1internal
// Порядок: Sandbox → Daily → Prod (Issue #1176)
// ---------------------------------------------------------------------------

const V1_INTERNAL_BASE_URL_PROD: &str =
    "https://cloudcode-pa.googleapis.com/v1internal";
const V1_INTERNAL_BASE_URL_DAILY: &str =
    "https://daily-cloudcode-pa.googleapis.com/v1internal";
const V1_INTERNAL_BASE_URL_SANDBOX: &str =
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal";

const V1_INTERNAL_BASE_URL_FALLBACKS: [&str; 3] = [
    V1_INTERNAL_BASE_URL_SANDBOX,
    V1_INTERNAL_BASE_URL_DAILY,
    V1_INTERNAL_BASE_URL_PROD,
];

// ---------------------------------------------------------------------------
// UpstreamClient
// ---------------------------------------------------------------------------

pub struct UpstreamClient {
    default_client: Client,
    proxy_pool: Option<Arc<crate::proxy::proxy_pool::ProxyPoolManager>>,
    client_cache: DashMap<String, Client>,
    user_agent_override: RwLock<Option<String>>,
}

impl UpstreamClient {
    pub fn new(
        proxy_config: Option<crate::proxy::config::UpstreamProxyConfig>,
        proxy_pool: Option<Arc<crate::proxy::proxy_pool::ProxyPoolManager>>,
    ) -> Self {
        let default_client = match Self::build_client_internal(proxy_config) {
            Ok(client) => client,
            Err(err_with_proxy) => {
                tracing::error!(
                    error = %err_with_proxy,
                    "Failed to create HTTP client with proxy; retrying without proxy"
                );
                match Self::build_client_internal(None) {
                    Ok(client) => client,
                    Err(err_without_proxy) => {
                        tracing::error!(
                            error = %err_without_proxy,
                            "Failed to create HTTP client without proxy; using bare client"
                        );
                        crate::utils::http::get_client()
                    }
                }
            }
        };

        Self {
            default_client,
            proxy_pool,
            client_cache: DashMap::new(),
            user_agent_override: RwLock::new(None),
        }
    }

    /// Строит reqwest-клиент с опциональным upstream-прокси.
    /// ИСПРАВЛЕНО: возвращает reqwest::Error (в оригинале был rquest::Error — не компилировалось)
    fn build_client_internal(
        proxy_config: Option<crate::proxy::config::UpstreamProxyConfig>,
    ) -> Result<Client, reqwest::Error> {
        let mut builder = Client::builder()
            // Нет .emulation() — rquest не поддерживается на Android
            .connect_timeout(Duration::from_secs(20))
            .pool_max_idle_per_host(16)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(600));

        builder = Self::apply_default_user_agent(builder);

        if let Some(config) = proxy_config {
            if config.enabled && !config.url.is_empty() {
                let url = crate::proxy::config::normalize_proxy_url(&config.url);
                // ИСПРАВЛЕНО: убран невалидный синтаксис match #[cfg(...)] rquest::Proxy::all
                if let Ok(proxy) = Proxy::all(&url) {
                    builder = builder.proxy(proxy);
                    tracing::info!("UpstreamClient enabled proxy: {}", url);
                }
            }
        }

        builder.build()
    }

    /// Строит клиент с конкретным прокси из ProxyPool
    fn build_client_with_proxy(
        &self,
        proxy_config: crate::proxy::proxy_pool::PoolProxyConfig,
    ) -> Result<Client, reqwest::Error> {
        Client::builder()
            .connect_timeout(Duration::from_secs(20))
            .pool_max_idle_per_host(16)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(600))
            .proxy(proxy_config.proxy)
            .build()
    }

    fn apply_default_user_agent(builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        let ua = crate::constants::USER_AGENT.as_str();
        if header::HeaderValue::from_str(ua).is_ok() {
            builder.user_agent(ua)
        } else {
            tracing::warn!(
                user_agent = %ua,
                "Invalid default User-Agent value, using fallback"
            );
            builder.user_agent("antigravity")
        }
    }

    // -----------------------------------------------------------------------
    // User-Agent override
    // -----------------------------------------------------------------------

    pub async fn set_user_agent_override(&self, ua: Option<String>) {
        let mut lock = self.user_agent_override.write().await;
        *lock = ua;
        tracing::debug!("UpstreamClient User-Agent override updated: {:?}", lock);
    }

    pub async fn get_user_agent(&self) -> String {
        let ua_override = self.user_agent_override.read().await;
        ua_override
            .as_ref()
            .cloned()
            .unwrap_or_else(|| crate::constants::USER_AGENT.clone())
    }

    // -----------------------------------------------------------------------
    // Выбор клиента по account_id (ProxyPool)
    // -----------------------------------------------------------------------

    pub async fn get_client(&self, account_id: Option<&str>) -> Client {
        if let Some(pool) = &self.proxy_pool {
            if let Some(acc_id) = account_id {
                match pool.get_proxy_for_account(acc_id).await {
                    Ok(Some(proxy_cfg)) => {
                        if let Some(client) = self.client_cache.get(&proxy_cfg.entry_id) {
                            return client.clone();
                        }
                        match self.build_client_with_proxy(proxy_cfg.clone()) {
                            Ok(client) => {
                                self.client_cache
                                    .insert(proxy_cfg.entry_id.clone(), client.clone());
                                tracing::info!(
                                    "Using ProxyPool proxy ID: {} for account: {}",
                                    proxy_cfg.entry_id,
                                    acc_id
                                );
                                return client;
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to build client for proxy {}: {}, using default",
                                    proxy_cfg.entry_id,
                                    e
                                );
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        tracing::error!(
                            "Error getting proxy for account {}: {}, using default",
                            acc_id,
                            e
                        );
                    }
                }
            }
        }
        self.default_client.clone()
    }

    // -----------------------------------------------------------------------
    // URL-строитель
    // -----------------------------------------------------------------------

    fn build_url(base_url: &str, method: &str, query_string: Option<&str>) -> String {
        match query_string {
            Some(qs) => format!("{}:{}?{}", base_url, method, qs),
            None => format!("{}:{}", base_url, method),
        }
    }

    // -----------------------------------------------------------------------
    // Логика fallback-переключения
    // -----------------------------------------------------------------------

    fn should_try_next_endpoint(status: StatusCode) -> bool {
        status == StatusCode::TOO_MANY_REQUESTS
            || status == StatusCode::REQUEST_TIMEOUT
            || status == StatusCode::NOT_FOUND
            || status.is_server_error()
    }

    // -----------------------------------------------------------------------
    // Основные методы вызова v1internal API
    // -----------------------------------------------------------------------

    pub async fn call_v1_internal(
        &self,
        method: &str,
        access_token: &str,
        body: Value,
        query_string: Option<&str>,
        account_id: Option<&str>,
    ) -> Result<UpstreamCallResult, String> {
        self.call_v1_internal_with_headers(
            method,
            access_token,
            body,
            query_string,
            std::collections::HashMap::new(),
            account_id,
        )
        .await
    }

    /// FIX #765: вызов с поддержкой extra-заголовков (например anthropic-beta)
    pub async fn call_v1_internal_with_headers(
        &self,
        method: &str,
        access_token: &str,
        body: Value,
        query_string: Option<&str>,
        extra_headers: std::collections::HashMap<String, String>,
        account_id: Option<&str>,
    ) -> Result<UpstreamCallResult, String> {
        let client = self.get_client(account_id).await;

        // --- Базовые заголовки ---
        let mut headers = header::HeaderMap::new();

        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| e.to_string())?,
        );

        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&self.get_user_agent().await).unwrap_or_else(
                |e| {
                    tracing::warn!("Invalid User-Agent, using fallback: {}", e);
                    header::HeaderValue::from_static("antigravity")
                },
            ),
        );

        // --- Client Identity ---
        headers.insert(
            "x-client-name",
            header::HeaderValue::from_static("antigravity"),
        );
        if let Ok(ver) =
            header::HeaderValue::from_str(&crate::constants::CURRENT_VERSION)
        {
            headers.insert("x-client-version", ver);
        }

        // --- Device & Session Identity ---
        // ИСПРАВЛЕНО: machine_uid на Android часто пустой — добавлен fallback
        let machine_id = Self::get_machine_id();
        if !machine_id.is_empty() {
            if let Ok(mid_val) = header::HeaderValue::from_str(&machine_id) {
                headers.insert("x-machine-id", mid_val);
            }
        }

        if let Ok(sess_val) =
            header::HeaderValue::from_str(&crate::constants::SESSION_ID)
        {
            headers.insert("x-vscode-sessionid", sess_val);
        }

        // x-goog-api-client намеренно убран (v4.1.24) — не восстанавливаем

        // --- Extra Headers ---
        for (k, v) in extra_headers {
            if let Ok(hk) = header::HeaderName::from_bytes(k.as_bytes()) {
                if let Ok(hv) = header::HeaderValue::from_str(&v) {
                    headers.insert(hk, hv);
                }
            }
        }

        tracing::debug!(?headers, "Final Upstream Request Headers");

        // --- Fallback по эндпоинтам ---
        let mut last_err: Option<String> = None;
        let mut fallback_attempts: Vec<FallbackAttemptLog> = Vec::new();

        for (idx, base_url) in V1_INTERNAL_BASE_URL_FALLBACKS.iter().enumerate() {
            let url = Self::build_url(base_url, method, query_string);
            let has_next = idx + 1 < V1_INTERNAL_BASE_URL_FALLBACKS.len();

            let response = client
                .post(&url)
                .headers(headers.clone())
                .json(&body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        if idx > 0 {
                            tracing::info!(
                                "✓ Upstream fallback succeeded | Endpoint: {} | Status: {}",
                                base_url,
                                status
                            );
                        } else {
                            tracing::debug!(
                                "✓ Upstream succeeded | Endpoint: {} | Status: {}",
                                base_url,
                                status
                            );
                        }
                        return Ok(UpstreamCallResult {
                            response: resp,
                            fallback_attempts,
                        });
                    }

                    if has_next && Self::should_try_next_endpoint(status) {
                        let err_msg =
                            format!("Upstream {} returned {}", base_url, status);
                        tracing::warn!(
                            "Endpoint {} returned {} (method={}), trying next",
                            base_url,
                            status,
                            method
                        );
                        fallback_attempts.push(FallbackAttemptLog {
                            endpoint_url: url,
                            status: Some(status.as_u16()),
                            error: err_msg.clone(),
                        });
                        last_err = Some(err_msg);
                        continue;
                    }

                    return Ok(UpstreamCallResult {
                        response: resp,
                        fallback_attempts,
                    });
                }
                Err(e) => {
                    let msg =
                        format!("HTTP request failed at {}: {}", base_url, e);
                    tracing::debug!("{}", msg);
                    fallback_attempts.push(FallbackAttemptLog {
                        endpoint_url: url,
                        status: None,
                        error: msg.clone(),
                    });
                    last_err = Some(msg);
                    if !has_next {
                        break;
                    }
                    continue;
                }
            }
        }

        Err(last_err.unwrap_or_else(|| "All endpoints failed".to_string()))
    }

    // -----------------------------------------------------------------------
    // Machine ID с fallback для Android
    // ИСПРАВЛЕНО: на Android machine_uid::get() часто возвращает пустую строку
    // Решение: пробуем machine_uid → читаем/создаём кэш-файл с UUID
    // -----------------------------------------------------------------------

    fn get_machine_id() -> String {
        // Попытка 1: machine_uid (работает на десктопе, иногда на Android)
        if let Ok(mid) = machine_uid::get() {
            if !mid.is_empty() {
                return mid;
            }
        }

        // Попытка 2: кэшированный UUID в файле (персистентный между запусками)
        let cache_path = {
            // На Android data-dir передаётся через env из Tauri mobile runtime
            let base = std::env::var("APP_DATA_DIR")
                .or_else(|_| std::env::var("ANTIGRAVITY_DATA_DIR"))
                .unwrap_or_else(|_| {
                    // fallback для Termux/dev окружения
                    dirs::data_local_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "/tmp".to_string())
                });
            format!("{}/.machine_id", base)
        };

        if let Ok(cached) = std::fs::read_to_string(&cache_path) {
            let trimmed = cached.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }

        // Генерируем новый UUID и сохраняем
        let new_id = uuid::Uuid::new_v4().to_string();
        if let Some(parent) = std::path::Path::new(&cache_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(&cache_path, &new_id).is_ok() {
            tracing::info!(
                "machine_uid unavailable; generated persistent UUID: {}",
                new_id
            );
        }
        new_id
    }

    // -----------------------------------------------------------------------
    // fetch_available_models (сохранён из оригинала)
    // -----------------------------------------------------------------------

    #[allow(dead_code)]
    pub async fn fetch_available_models(
        &self,
        access_token: &str,
        account_id: Option<&str>,
    ) -> Result<Value, String> {
        let result = self
            .call_v1_internal(
                "fetchAvailableModels",
                access_token,
                serde_json::json!({}),
                None,
                account_id,
            )
            .await?;
        let json: Value = result
            .response
            .json()
            .await
            .map_err(|e| format!("Parse json failed: {}", e))?;
        Ok(json)
    }
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_no_query() {
        let base = "https://cloudcode-pa.googleapis.com/v1internal";
        let url = UpstreamClient::build_url(base, "generateContent", None);
        assert_eq!(
            url,
            "https://cloudcode-pa.googleapis.com/v1internal:generateContent"
        );
    }

    #[test]
    fn test_build_url_with_query() {
        let base = "https://cloudcode-pa.googleapis.com/v1internal";
        let url = UpstreamClient::build_url(
            base,
            "streamGenerateContent",
            Some("alt=sse"),
        );
        assert_eq!(
            url,
            "https://cloudcode-pa.googleapis.com/v1internal:streamGenerateContent?alt=sse"
        );
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("userexample@gmail.com"), "use***@gm***");
        assert_eq!(mask_email("nodomain"), "noDom***");
    }

    #[test]
    fn test_should_try_next_endpoint() {
        assert!(UpstreamClient::should_try_next_endpoint(
            StatusCode::TOO_MANY_REQUESTS
        ));
        assert!(UpstreamClient::should_try_next_endpoint(
            StatusCode::INTERNAL_SERVER_ERROR
        ));
        assert!(!UpstreamClient::should_try_next_endpoint(
            StatusCode::UNAUTHORIZED
        ));
        assert!(!UpstreamClient::should_try_next_endpoint(StatusCode::OK));
    }
}
