use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioResolver,
    name_server::TokioConnectionProvider,
};

pub async fn resolve_hostname(hostname: &str) -> anyhow::Result<String> {
    let resolver = TokioResolver::builder_with_config(
        ResolverConfig::cloudflare(),
        ResolverOpts::default(),
    ).build();

    let response = resolver
        .lookup_ip(hostname)
        .await
        .map_err(|e| anyhow::anyhow!("DoH lookup failed: {}", e))?;

    let addr = response
        .iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No IP found for {}", hostname))?;

    Ok(addr.to_string())
}
