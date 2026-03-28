// src-tauri/src/utils/http.rs
// Android-адаптация: rquest → reqwest (reqwest уже есть в оригинальном Cargo.toml)
// JA3/TLS эмуляция убрана — rquest не компилируется под aarch64-linux-android

#[cfg(not(target_os = "android"))]
use crate::modules::config::load_app_config;
use once_cell::sync::Lazy;
use reqwest::{Client, Proxy};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Глобальные shared-клиенты
// clone() клиента — лёгкая операция, connection pool переиспользуется
// ---------------------------------------------------------------------------

/// Основной клиент (15 с таймаут)
pub static SHARED_CLIENT: Lazy<Client> = Lazy::new(|| create_base_client(15));

/// Клиент с длинным таймаутом (60 с) — для warmup и long-poll запросов
pub static SHARED_CLIENT_LONG: Lazy<Client> = Lazy::new(|| create_base_client(60));

/// Стандартный клиент (15 с) — алиас для совместимости с десктопным API
pub static SHARED_STANDARD_CLIENT: Lazy<Client> = Lazy::new(|| create_base_client(15));

/// Стандартный клиент (60 с)
pub static SHARED_STANDARD_CLIENT_LONG: Lazy<Client> = Lazy::new(|| create_base_client(60));

// ---------------------------------------------------------------------------
// Внутренняя логика создания клиента
// ---------------------------------------------------------------------------

fn create_base_client(timeout_secs: u64) -> Client {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .user_agent(if cfg!(target_os = "android") { "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36" } else { "Mozilla/5.0" });

    #[cfg(not(target_os = "android"))]
if let Ok(config) = load_app_config() {
        let proxy_cfg = config.proxy.upstream_proxy;
        if proxy_cfg.enabled && !proxy_cfg.url.is_empty() {
            match Proxy::all(&proxy_cfg.url) {
                Ok(proxy) => {
                    builder = builder.proxy(proxy);
                    tracing::info!(
                        "HTTP client enabled upstream proxy: {}",
                        proxy_cfg.url
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Invalid upstream proxy URL '{}': {}",
                        proxy_cfg.url,
                        e
                    );
                }
            }
        }
    }

    tracing::info!(
        "Initialized reqwest client (timeout={}s) [Android — no JA3/TLS emulation]",
        timeout_secs
    );

    builder.build().unwrap_or_else(|e| {
        tracing::error!("Failed to build HTTP client: {}, using bare client", e);
        Client::new()
    })
}

// ---------------------------------------------------------------------------
// Публичные хелперы (совместимый API с десктопной версией)
// ---------------------------------------------------------------------------

/// Основной клиент (15 с)
pub fn get_client() -> Client {
    SHARED_CLIENT.clone()
}

/// Клиент с длинным таймаутом (60 с)
pub fn get_long_client() -> Client {
    SHARED_CLIENT_LONG.clone()
}

/// Стандартный клиент без JA3 (на Android все клиенты такие)
pub fn get_standard_client() -> Client {
    SHARED_STANDARD_CLIENT.clone()
}

/// Стандартный клиент с длинным таймаутом
pub fn get_long_standard_client() -> Client {
    SHARED_STANDARD_CLIENT_LONG.clone()
}

// === STEALTH CLIENT (BoringSSL + Dynamic Chrome fingerprint) ===
#[cfg(target_os = "android")]
pub mod stealth {
    use boring::ssl::{SslConnector, SslMethod};
    use hyper_boring::HttpsConnector;
    use hyper014::Client;
    use crate::utils::fingerprint::FingerprintConfig;

    pub type StealthClient = Client<HttpsConnector<hyper014::client::HttpConnector>>;

    pub fn get_stealth_client() -> anyhow::Result<StealthClient> {
        let fp = FingerprintConfig::current();
        let mut builder = SslConnector::builder(SslMethod::tls_client())?;

        builder.set_grease_enabled(true);
        builder.enable_ocsp_stapling();
        // TLS Session Ticket control — отключаем resumption для избежания паттернов
        builder.set_session_cache_mode(boring::ssl::SslSessionCacheMode::OFF);
        builder.set_cipher_list(&fp.cipher_list)?;
        builder.set_alpn_protos(b"\x02h2\x08http/1.1")?;
        builder.enable_signed_cert_timestamps();
        builder.set_curves_list("X25519:P-256:P-384")?;

        let mut http = hyper014::client::HttpConnector::new();
        http.enforce_http(false);

        let connector = HttpsConnector::with_connector(http, builder)?;
        // HTTP/2 fingerprinting — Chrome SETTINGS frames
        Ok(Client::builder()
            .http2_initial_stream_window_size(6291456u32)
            .http2_initial_connection_window_size(15728640u32)
            .http2_max_frame_size(16384u32)
            .build(connector))
    }
}
