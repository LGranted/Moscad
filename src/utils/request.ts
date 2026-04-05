import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import type { Account, AccountQuota, ProxyStatus, LogEntry, AppConfig, OAuthStartResult } from '../types';

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args);
  } catch (err) {
    if (typeof err === 'string') throw new Error(err);
    throw err;
  }
}

export { invoke as request };

// Accounts
export const getAccounts = () => invoke<Account[]>('list_accounts');
export const getCurrentAccount = () => invoke<Account | null>('get_current_account');
export const addAccount = (email: string, refreshToken: string) =>
  invoke<Account>('add_account', { email, refreshToken });
export const deleteAccount = (accountId: string) =>
  invoke<void>('delete_account', { accountId });
export const deleteAccounts = (accountIds: string[]) =>
  invoke<void>('delete_accounts', { accountIds });
export const switchAccount = (accountId: string) =>
  invoke<void>('switch_account', { accountId });
export const fetchQuota = (accountId: string) =>
  invoke<AccountQuota>('fetch_account_quota', { accountId });
export const refreshAccountQuota = (accountId: string) =>
  invoke<void>('refresh_account_quota', { accountId });
export const refreshAllQuotas = () => invoke<void>('refresh_all_quotas');
export const reorderAccounts = (accountIds: string[]) =>
  invoke<void>('reorder_accounts', { accountIds });
export const toggleProxyStatus = (accountId: string) =>
  invoke<void>('toggle_proxy_status', { accountId });
export const warmUpAllAccounts = () => invoke<void>('warm_up_all_accounts');
export const warmUpAccount = (accountId: string) =>
  invoke<void>('warm_up_account', { accountId });

// OAuth
export const startOAuthFlow = () => invoke<OAuthStartResult>('prepare_oauth_url');
export const submitOAuthCode = (code: string, state: string | null) =>
  invoke<void>('submit_oauth_code', { code, state });
export const cancelOAuthFlow = () => invoke<void>('cancel_oauth_login');
export const completeOAuthLogin = (code: string) =>
  invoke<void>('complete_oauth_login', { code });

// Proxy
export const startProxyService = () => invoke<void>('start_proxy_service');
export const stopProxyService = () => invoke<void>('stop_proxy_service');
export const getProxyStatus = () => invoke<ProxyStatus>('get_proxy_status');
export const getProxyStats = () => invoke<Record<string, unknown>>('get_proxy_stats');
export const clearProxyRateLimit = (accountId: string) =>
  invoke<void>('clear_proxy_rate_limit', { accountId });
export const clearAllProxyRateLimits = () => invoke<void>('clear_all_proxy_rate_limits');
export const clearProxyLogs = () => invoke<void>('clear_proxy_logs');
export const checkProxyHealth = () => invoke<boolean>('check_proxy_health');
export const getPreferredAccount = () => invoke<string | null>('get_preferred_account');
export const setPreferredAccount = (accountId: string) =>
  invoke<void>('set_preferred_account', { accountId });
export const clearProxySessionBindings = () => invoke<void>('clear_proxy_session_bindings');

// Logs
export const getLogsFiltered = (errorsOnly: boolean, search: string, limit: number) =>
  invoke<LogEntry[]>('get_proxy_logs_filtered', { errorsOnly, search, limit });
export const getLogsCount = (errorsOnly: boolean, search: string) =>
  invoke<number>('get_proxy_logs_count_filtered', { errorsOnly, search });
export const getLogDetail = (logId: string) =>
  invoke<LogEntry>('get_proxy_log_detail', { logId });
export const exportProxyLogs = () => invoke<string>('export_proxy_logs');
export const exportProxyLogsJson = () => invoke<string>('export_proxy_logs_json');

// Config
export const loadConfig = () => invoke<AppConfig>('load_config');
export const saveConfig = (config: AppConfig) => invoke<void>('save_config', { config });

// Cloudflared
export const cloudflaredInstall = () => invoke<void>('cloudflared_install');
export const cloudflaredStart = () => invoke<void>('cloudflared_start');
export const cloudflaredStop = () => invoke<void>('cloudflared_stop');
export const cloudflaredGetStatus = () => invoke<Record<string, unknown>>('cloudflared_get_status');
