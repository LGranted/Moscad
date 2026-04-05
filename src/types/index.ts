// ─── Account ───────────────────────────────────────────────────────────────
export interface Account {
  id: string;
  email: string;
  label?: string;
  machine_id: string;
  is_current: boolean;
  disabled: boolean;
  disabled_reason?: string;
  created_at: number;
  last_used: number;
}

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

export interface AggregatedQuota {
  model: string;
  total_requests_remaining: number;
  total_tokens_remaining: number;
  accounts_count: number;
}

export interface ProxyStatus {
  is_running: boolean;
  port: number;
}

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
}

export interface LogFilter {
  errors_only: boolean;
  search: string;
  limit: number;
}

export interface AppConfig {
  language: string;
  theme: 'light' | 'dark' | 'system';
  proxy_port: number;
  proxy_host: string;
  api_key: string;
  log_enabled: boolean;
  log_level: 'debug' | 'info' | 'warn' | 'error';
  rotation_enabled: boolean;
  auto_start_proxy: boolean;
  retry_enabled: boolean;
  retry_limit: number;
  retry_base_delay_ms: number;
  request_timeout_ms: number;
}

export const DEFAULT_CONFIG: AppConfig = {
  language: 'ru',
  theme: 'system',
  proxy_port: 8045,
  proxy_host: '127.0.0.1',
  api_key: '',
  log_enabled: true,
  log_level: 'info',
  rotation_enabled: false,
  auto_start_proxy: false,
  retry_enabled: true,
  retry_limit: 3,
  retry_base_delay_ms: 1000,
  request_timeout_ms: 30000,
};

export interface OAuthStartResult {
  url: string;
  state: string;
}
