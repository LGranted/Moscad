#[cfg(target_os = "android")]
use hickory_resolver::{
    TokioAsyncResolver,
    config::{ResolverConfig, ResolverOpts},
};

#[cfg(target_os = "android")]
pub async fn resolve_with_doh(hostname: &str) -> Option<std::net::IpAddr> {
    let mut opts = ResolverOpts::default();
    opts.cache_size = 32;

    // Используем Cloudflare DoH
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::cloudflare(),
        opts,
    );

    resolver
        .lookup_ip(hostname)
        .await
        .ok()?
        .iter()
        .next()
}
