use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    name_server::{GenericConnector, TokioRuntimeProvider},
    TokioResolver,
};

pub async fn resolve_hostname(hostname: &str) -> anyhow::Result<String> {
    let resolver = TokioResolver::builder_with_config(
        ResolverConfig::cloudflare(),
        GenericConnector::new(TokioRuntimeProvider::new()),
    )
    .with_options(ResolverOpts::default())
    .build();

    let response = resolver
        .lookup_ip(hostname)
        .await
        .map_err(|e| anyhow::anyhow!("DoH lookup failed for {}: {}", hostname, e))?;

    let addr = response
        .iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No IP resolved for {}", hostname))?;

    Ok(addr.to_string())
}
