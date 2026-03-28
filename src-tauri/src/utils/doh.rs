#[cfg(target_os = "android")]
use hickory_resolver::{
    TokioAsyncResolver,
    config::{ResolverConfig, ResolverOpts, NameServerConfig, Protocol},
};
#[cfg(target_os = "android")]
use std::net::SocketAddr;

#[cfg(target_os = "android")]
pub async fn resolve_with_doh(hostname: &str) -> Option<std::net::IpAddr> {
    let mut config = ResolverConfig::new();
    
    // Cloudflare DoH
    let socket_addr: SocketAddr = "1.1.1.1:443".parse().ok()?;
    let ns = NameServerConfig::new(socket_addr, Protocol::Https);
    config.add_name_server(ns);

    // Google DoH fallback  
    let socket_addr2: SocketAddr = "8.8.8.8:443".parse().ok()?;
    let ns2 = NameServerConfig::new(socket_addr2, Protocol::Https);
    config.add_name_server(ns2);

    let mut opts = ResolverOpts::default();
    opts.cache_size = 32;

    let resolver = TokioAsyncResolver::tokio(config, opts);
    
    resolver
        .lookup_ip(hostname)
        .await
        .ok()?
        .iter()
        .next()
}
