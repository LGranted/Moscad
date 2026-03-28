use once_cell::sync::OnceCell;

static FINGERPRINT: OnceCell<FingerprintConfig> = OnceCell::new();

pub struct FingerprintConfig {
    pub chrome_version: u32,
    pub user_agent: String,
    pub cipher_list: String,
}

impl FingerprintConfig {
    pub fn current() -> &'static FingerprintConfig {
        FINGERPRINT.get_or_init(|| {
            let base_version = 131u32;
            let base_timestamp = 1730000000u64;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let weeks_passed = (now.saturating_sub(base_timestamp)) / (4 * 7 * 24 * 3600);
            let version = (base_version + weeks_passed as u32).min(145);

            let user_agent = format!(
                "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Mobile Safari/537.36",
                version
            );

            FingerprintConfig {
                chrome_version: version,
                user_agent,
                cipher_list: "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305".to_string(),
            }
        })
    }
}
