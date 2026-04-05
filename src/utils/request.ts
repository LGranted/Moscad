import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Re-export invoke with typed error handling
export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args);
  } catch (err) {
    // Normalise Tauri errors to a regular Error
    if (typeof err === 'string') throw new Error(err);
    throw err;
  }
}

// ─── Typed wrappers ──────────────────────────────────────────────────────────
import type {
  Account,
  AccountQuota,
  ProxyStatus,
  LogEntry,
  AppConfig,
  OAuthStartResult,
} from '../types';

// Accounts
export const getAccounts = () => invoke<Account[]>('get_accounts');
export const addAccount = (email: string, token: string) =>
  invoke<void>('add_account', { email, token });
export const deleteAccount = (accountId: string) =>
  invoke<void>('delete_account', { accountId });
export const setCurrentAccount = (accountId: string) =>
  invoke<void>('set_current_account', { accountId });
export const rotateAccount = () => invoke<void>('rotate_account');
export const clearProxyRateLimit = (accountId: string) =>
  invoke<void>('clear_proxy_rate_limit', { accountId });
export const clearAllProxyRateLimits = () =>
  invoke<void>('clear_all_proxy_rate_limits');
export const fetchQuota = (accountId: string) =>
  invoke<AccountQuota>('fetch_quota', { accountId });

// Proxy
export const startProxyService = () => invoke<void>('start_proxy_service');
export const stopProxyService = () => invoke<void>('stop_proxy_service');
export const getProxyStatus = () => invoke<ProxyStatus>('get_proxy_status');

// Logs
export const getLogs = (limit?: number) =>
  invoke<LogEntry[]>('get_logs', { limit: limit ?? 100 });
export const getLogsFiltered = (errorsOnly: boolean, search: string, limit: number) =>
  invoke<LogEntry[]>('get_logs_filtered', { errorsOnly, search, limit });
export const getLogsCount = () => invoke<number>('get_logs_count');

// Config
export const loadConfig = () => invoke<AppConfig>('load_config');
export const saveConfig = (config: AppConfig) => invoke<void>('save_config', { config });

// OAuth
export const startOAuthFlow = () => invoke<OAuthStartResult>('start_oauth_flow');
export const submitOAuthCode = (code: string, state: string) =>
  invoke<void>('submit_oauth_code', { code, state });
export const cancelOAuthFlow = () => invoke<void>('cancel_oauth_flow');
