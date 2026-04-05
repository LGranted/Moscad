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

// Accounts
export const getAccounts = () => invoke<Account[]>('list_accounts');
export const addAccount = (email: string, refresh_token: string) =>
  invoke<Account>('add_account', { email, refreshToken: refresh_token });
export const deleteAccount = (account_id: string) =>
  invoke<void>('delete_account', { accountId: account_id });
export const switchAccount = (account_id: string) =>
  invoke<void>('switch_account', { accountId: account_id });
export const clearProxyRateLimit = (account_id: string) =>
  invoke<void>('clear_proxy_rate_limit', { accountId: account_id });
export const clearAllProxyRateLimits = () =>
  invoke<void>('clear_all_proxy_rate_limits');
export const fetchQuota = (account_id: string) =>
  invoke<AccountQuota>('fetch_account_quota', { accountId: account_id });

// Proxy
export const startProxyService = () => invoke<void>('start_proxy_service');
export const stopProxyService = () => invoke<void>('stop_proxy_service');
export const getProxyStatus = () => invoke<ProxyStatus>('get_proxy_status');

// Logs
export const getLogsFiltered = (errorsOnly: boolean, search: string, limit: number) =>
  invoke<LogEntry[]>('get_proxy_logs_filtered', { errorsOnly, search, limit });
export const getLogsCount = (errorsOnly: boolean, search: string) =>
  invoke<number>('get_proxy_logs_count_filtered', { errorsOnly, search });

// Config
export const loadConfig = () => invoke<AppConfig>('load_config');
export const saveConfig = (config: AppConfig) => invoke<void>('save_config', { config });

// OAuth
export const startOAuthFlow = () => invoke<OAuthStartResult>('prepare_oauth_url');
export const submitOAuthCode = (code: string, state: string | null) =>
  invoke<void>('submit_oauth_code', { code, state });
export const cancelOAuthFlow = () => invoke<void>('cancel_oauth_login');
