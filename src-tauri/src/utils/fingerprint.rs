use once_cell::sync::OnceCell;

static FINGERPRINT: OnceCell<FingerprintConfig> = OnceCell::new();

pub struct FingerprintConfig {
    pub chrome_version: u32,
    pub user_agent: String,
    pub antigravity_version: String,
    pub cipher_list: String,
}

impl FingerprintConfig {
    pub fn current() -> &'static FingerprintConfig {
        FINGERPRINT.get_or_init(|| {
            // Antigravity версия — динамически обновляется каждые 2 недели
            // База: 1.21.6 (март 2026)
            let base_ag_minor = 21u32;
            let base_ag_patch = 6u32;
            let base_timestamp = 1741000000u64; // март 2026

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let weeks_passed = (now.saturating_sub(base_timestamp)) / (2 * 7 * 24 * 3600);
            let ag_patch = base_ag_patch + weeks_passed as u32;
            let ag_version = format!("1.{}.{}", base_ag_minor, ag_patch);

            // User-Agent имитирует десктопный Antigravity клиент
            // Формат: antigravity/<version> <platform>/<arch> google-api-nodejs-client/<lib_version>
            let user_agent = format!(
                "antigravity/{} darwin/arm64",
                ag_version
            );

            // Chrome версия для TLS fingerprint (не для UA)
            let base_chrome = 131u32;
            let base_chrome_ts = 1730000000u64;
            let chrome_weeks = (now.saturating_sub(base_chrome_ts)) / (4 * 7 * 24 * 3600);
            let chrome_version = (base_chrome + chrome_weeks as u32).min(145);

            FingerprintConfig {
                chrome_version,
                user_agent,
                antigravity_version: ag_version,
                cipher_list: "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305".to_string(),
            }
        })
    }
}
