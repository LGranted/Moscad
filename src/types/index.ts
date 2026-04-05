// ─── Account ───────────────────────────────────────────────────────────────
export interface Account {
  id: string;
  email: string;
  is_valid: boolean;
  is_current: boolean;
  added_at?: string;
}

// ─── Quota ──────────────────────────────────────────────────────────────────
export interface ModelQuota {
  model: string;
  requests_remaining: number | null;
  tokens_remaining: number | null;
  requests_limit: number | null;
  tokens_limit: number | null;
  reset_at?: string;
}

export interface AccountQuota {
  account_id: string;
  email: string;
  quotas: ModelQuota[];
  error?: string;
}

// Aggregated across all accounts per model
export interface AggregatedQuota {
  model: string;
  total_requests_remaining: number;
  total_tokens_remaining: number;
  accounts_count: number;
}

// ─── Proxy ──────────────────────────────────────────────────────────────────
export interface ProxyStatus {
  is_running: boolean;
  address: string;
  api_key: string;
  port: number;
}

// ─── Logs ───────────────────────────────────────────────────────────────────
export interface LogEntry {
  id: string;
  timestamp: string;
  method: string;
  url: string;
  status_code: number;
  duration_ms: number;
  account_email?: string;
  model?: string;
  error?: string;
  request_body?: string;
  response_body?: string;
}

export interface LogFilter {
  errors_only: boolean;
  search: string;
  limit: number;
}

// ─── Config ──────────────────────────────────────────────────────────────────
export interface AppConfig {
  language: string;
  theme: 'light' | 'dark' | 'system';
  proxy_port: number;
  proxy_host: string;
  api_key: string;
  log_enabled: boolean;
  log_level: 'debug' | 'info' | 'warn' | 'error';
  rotation_enabled: boolean;
  rotation_strategy: 'round-robin' | 'by-limit';
  auto_start_proxy: boolean;
  retry_enabled: boolean;
  retry_limit: number;
  retry_base_delay_ms: number;
  upstream_proxy?: string;
  request_timeout_ms: number;
}

export const DEFAULT_CONFIG: AppConfig = {
  language: 'ru',
  theme: 'system',
  proxy_port: 8080,
  proxy_host: '127.0.0.1',
  api_key: 'test',
  log_enabled: true,
  log_level: 'info',
  rotation_enabled: false,
  rotation_strategy: 'round-robin',
  auto_start_proxy: false,
  retry_enabled: true,
  retry_limit: 3,
  retry_base_delay_ms: 1000,
  request_timeout_ms: 30000,
};

// ─── OAuth ───────────────────────────────────────────────────────────────────
export interface OAuthStartResult {
  url: string;
  state: string;
}
