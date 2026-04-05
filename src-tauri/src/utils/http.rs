// src-tauri/src/utils/http.rs
// Обычные клиенты: reqwest (совместимость с oauth.rs, quota.rs и др.)
// Stealth клиент: hyper014 через Unix Domain Socket → Go uTLS сайдкар

#[cfg(not(target_os = "android"))]
use crate::modules::config::load_app_config;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;
// DoH resolver for Android using hickory-resolver
#[cfg(target_os = "android")]
mod doh_resolver {
    use hickory_resolver::{
        config::{ResolverConfig, ResolverOpts},
        name_server::TokioConnectionProvider,
        TokioResolver,
    };
    use reqwest::dns::{Resolve, Resolving};
    use std::net::SocketAddr;

    #[derive(Clone)]
    pub struct DoHResolver {
        resolver: TokioResolver,
    }

    impl DoHResolver {
        pub fn new() -> Self {
            let resolver = TokioResolver::builder_with_config(
                ResolverConfig::cloudflare(),
                TokioConnectionProvider::default(),
            )
            .with_options(ResolverOpts::default())
            .build();
            Self { resolver }
        }
    }

    impl Resolve for DoHResolver {
        fn resolve(&self, name: reqwest::dns::Name) -> Resolving {
            let resolver = self.resolver.clone();
            let name_str = name.as_str().to_string();
            Box::pin(async move {
                let response = resolver.lookup_ip(&name_str).await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
                let addrs: Box<dyn Iterator<Item = SocketAddr> + Send> =
                    Box::new(response.iter().map(|ip| SocketAddr::new(ip, 0)));
                Ok(addrs)
            })
        }
    }
}


// ── Глобальные shared reqwest клиенты ────────────────────────────────────────

pub static SHARED_CLIENT: Lazy<Client> = Lazy::new(|| create_base_client(15));
pub static SHARED_CLIENT_LONG: Lazy<Client> = Lazy::new(|| create_base_client(60));
pub static SHARED_STANDARD_CLIENT: Lazy<Client> = Lazy::new(|| create_base_client(15));
pub static SHARED_STANDARD_CLIENT_LONG: Lazy<Client> = Lazy::new(|| create_base_client(60));

fn create_base_client(timeout_secs: u64) -> Client {
    let builder = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .local_address(Some(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)))
        .user_agent(crate::utils::fingerprint::FingerprintConfig::current().user_agent.clone());
    #[cfg(target_os = "android")]
    let builder = builder.dns_resolver(std::sync::Arc::new(doh_resolver::DoHResolver::new()));
    #[cfg(not(target_os = "android"))]
    let builder = {
        if let Ok(config) = crate::modules::config::load_app_config() {
            let proxy_cfg = config.proxy.upstream_proxy;
            if proxy_cfg.enabled && !proxy_cfg.url.is_empty() {
                match reqwest::Proxy::all(&proxy_cfg.url) {
                    Ok(proxy) => builder.proxy(proxy),
                    Err(e) => {
                        tracing::error!("Invalid proxy URL '{}': {}", proxy_cfg.url, e);
                        builder
                    }
                }
            } else {
                builder
            }
        } else {
            builder
        }
    };
    builder.build().unwrap_or_else(|_| Client::new())
}

pub fn get_client() -> Client { SHARED_CLIENT.clone() }
pub fn get_long_client() -> Client { SHARED_CLIENT_LONG.clone() }
pub fn get_standard_client() -> Client { SHARED_STANDARD_CLIENT.clone() }
pub fn get_long_standard_client() -> Client { SHARED_STANDARD_CLIENT_LONG.clone() }

// ── Stealth модуль: hyper014 → UDS → Go сайдкар (Chrome 131 TLS) ─────────────

#[cfg(target_os = "android")]
pub mod stealth {
    use std::path::PathBuf;
    use tokio::net::UnixStream;
    use hyper014::client::connect::{Connected, Connection};
    use hyper014::Uri;
    use std::future::Future;
    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    pub const UTLS_SOCK_PATH: &str =
        "/data/data/com.lbjlaq.antigravity/files/utls.sock";

    pub type StealthClient = hyper014::Client<UnixConnector, hyper014::Body>;

    #[derive(Clone)]
    pub struct UnixConnector {
        path: PathBuf,
    }

    impl tower::Service<Uri> for UnixConnector {
        type Response = UnixStreamWrapper;
        type Error = io::Error;
        type Future = Pin<Box<dyn Future<Output = io::Result<UnixStreamWrapper>> + Send + 'static>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _: Uri) -> Self::Future {
            let path = self.path.clone();
            Box::pin(async move { UnixStream::connect(path).await.map(UnixStreamWrapper) })
        }
    }

    pub struct UnixStreamWrapper(UnixStream);
    impl Connection for UnixStreamWrapper {
        fn connected(&self) -> Connected { Connected::new() }
    }
    impl tokio::io::AsyncRead for UnixStreamWrapper {
        fn poll_read(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> std::task::Poll<std::io::Result<()>> {
            std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
        }
    }
    impl tokio::io::AsyncWrite for UnixStreamWrapper {
        fn poll_write(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &[u8]) -> std::task::Poll<std::io::Result<usize>> {
            std::pin::Pin::new(&mut self.0).poll_write(cx, buf)
        }
        fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
            std::pin::Pin::new(&mut self.0).poll_flush(cx)
        }
        fn poll_shutdown(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
            std::pin::Pin::new(&mut self.0).poll_shutdown(cx)
        }
    }

    pub fn get_stealth_client() -> anyhow::Result<StealthClient> {
        get_stealth_client_for(None)
    }

    pub fn get_stealth_client_for(_account_seed: Option<&str>) -> anyhow::Result<StealthClient> {
        let connector = UnixConnector {
            path: PathBuf::from(UTLS_SOCK_PATH),
        };
        let client = hyper014::Client::builder()
            .pool_max_idle_per_host(8)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build::<_, hyper014::Body>(connector);
        Ok(client)
    }
}
