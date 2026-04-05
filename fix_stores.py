#!/usr/bin/env python3
import os

BASE = os.path.expanduser("~/Antigravity-Manager")

def write(path, content):
    full = os.path.join(BASE, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, 'w') as f:
        f.write(content)
    print(f"OK {path}")

# ── useAccountStore.ts ───────────────────────────────────────────────────────
write("src/stores/useAccountStore.ts", """
import { create } from 'zustand';
import type { Account, AccountQuota, AggregatedQuota } from '../types';
import {
  getAccounts,
  deleteAccount as apiDeleteAccount,
  switchAccount as apiSwitchAccount,
  clearProxyRateLimit as apiClearRateLimit,
  fetchQuota as apiFetchQuota,
  refreshAllQuotas as apiRefreshAll,
} from '../utils/request';

interface AccountState {
  accounts: Account[];
  quotas: Record<string, AccountQuota>;
  loading: boolean;
  error: string | null;

  fetchAccounts: () => Promise<void>;
  deleteAccount: (id: string) => Promise<void>;
  switchAccount: (id: string) => Promise<void>;
  clearRateLimit: (id: string) => Promise<void>;
  fetchQuotaForAll: () => Promise<void>;
  getAggregatedQuotas: () => AggregatedQuota[];
}

export const useAccountStore = create<AccountState>((set, get) => ({
  accounts: [],
  quotas: {},
  loading: false,
  error: null,

  fetchAccounts: async () => {
    set({ loading: true, error: null });
    try {
      const accounts = await getAccounts();
      set({ accounts, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  deleteAccount: async (id) => {
    try {
      await apiDeleteAccount(id);
      await get().fetchAccounts();
    } catch (e) { set({ error: String(e) }); }
  },

  switchAccount: async (id) => {
    try {
      await apiSwitchAccount(id);
      await get().fetchAccounts();
    } catch (e) { set({ error: String(e) }); }
  },

  clearRateLimit: async (id) => {
    try {
      await apiClearRateLimit(id);
    } catch (e) { set({ error: String(e) }); }
  },

  fetchQuotaForAll: async () => {
    const { accounts } = get();
    if (accounts.length === 0) return;

    // Try bulk refresh first
    try {
      await apiRefreshAll();
    } catch {}

    // Then fetch individual quotas
    const results: Record<string, AccountQuota> = {};
    await Promise.allSettled(
      accounts.map(async (acc) => {
        try {
          results[acc.id] = await apiFetchQuota(acc.id);
        } catch (e) {
          results[acc.id] = {
            account_id: acc.id,
            email: acc.email,
            quotas: [],
            error: String(e),
          };
        }
      })
    );
    set({ quotas: results });
  },

  getAggregatedQuotas: () => {
    const { quotas } = get();
    const map = new Map<string, AggregatedQuota>();
    Object.values(quotas).forEach((aq) => {
      if (!aq.quotas) return;
      aq.quotas.forEach((mq) => {
        const existing = map.get(mq.model);
        const reqRem = mq.requests_remaining ?? 0;
        const tokRem = mq.tokens_remaining ?? 0;
        if (existing) {
          existing.total_requests_remaining += reqRem;
          existing.total_tokens_remaining += tokRem;
          existing.accounts_count += 1;
        } else {
          map.set(mq.model, {
            model: mq.model,
            total_requests_remaining: reqRem,
            total_tokens_remaining: tokRem,
            accounts_count: 1,
          });
        }
      });
    });
    return Array.from(map.values()).sort((a, b) => a.model.localeCompare(b.model));
  },
}));
""")

# ── useProxyStore.ts ─────────────────────────────────────────────────────────
write("src/stores/useProxyStore.ts", """
import { create } from 'zustand';
import type { ProxyStatus } from '../types';
import { getProxyStatus, startProxyService, stopProxyService } from '../utils/request';

interface ProxyState {
  status: ProxyStatus;
  loading: boolean;
  error: string | null;
  fetchStatus: () => Promise<void>;
  start: () => Promise<void>;
  stop: () => Promise<void>;
}

const DEFAULT_STATUS: ProxyStatus = {
  is_running: false,
  address: '127.0.0.1',
  api_key: '',
  port: 8080,
};

export const useProxyStore = create<ProxyState>((set, get) => ({
  status: DEFAULT_STATUS,
  loading: false,
  error: null,

  fetchStatus: async () => {
    try {
      const status = await getProxyStatus();
      set({ status, error: null });
    } catch {
      // keep current status on error, don't reset to default
    }
  },

  start: async () => {
    set({ loading: true, error: null });
    try {
      await startProxyService();
    } catch (e) {
      set({ error: String(e) });
    } finally {
      // Always fetch real status after start attempt
      try {
        const status = await getProxyStatus();
        set({ status });
      } catch {}
      set({ loading: false });
    }
  },

  stop: async () => {
    set({ loading: true, error: null });
    try {
      await stopProxyService();
    } catch (e) {
      set({ error: String(e) });
    } finally {
      // Always fetch real status after stop attempt
      try {
        const status = await getProxyStatus();
        set({ status });
      } catch {
        // If we can't get status after stop, assume stopped
        set({ status: { ...DEFAULT_STATUS } });
      }
      set({ loading: false });
    }
  },
}));
""")

# ── useConfigStore.ts ────────────────────────────────────────────────────────
write("src/stores/useConfigStore.ts", """
import { create } from 'zustand';
import type { AppConfig } from '../types';
import { DEFAULT_CONFIG } from '../types';
import { loadConfig, saveConfig as apiSave } from '../utils/request';

interface ConfigState {
  config: AppConfig;
  loading: boolean;
  error: string | null;
  fetchConfig: () => Promise<void>;
  saveConfig: (partial: Partial<AppConfig>) => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set, get) => ({
  config: DEFAULT_CONFIG,
  loading: false,
  error: null,

  fetchConfig: async () => {
    set({ loading: true, error: null });
    try {
      const config = await loadConfig();
      const merged = { ...DEFAULT_CONFIG, ...config };
      set({ config: merged, loading: false });
      applyTheme(merged.theme ?? 'system');
    } catch {
      set({ loading: false });
      applyTheme('system');
    }
  },

  saveConfig: async (partial) => {
    const newConfig = { ...get().config, ...partial };
    set({ config: newConfig });
    if (partial.theme !== undefined) {
      applyTheme(partial.theme);
    }
    try {
      await apiSave(newConfig);
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));

function applyTheme(theme: AppConfig['theme']) {
  const root = document.documentElement;

  const apply = (dark: boolean) => {
    // Tailwind dark mode
    root.classList.toggle('dark', dark);
    // DaisyUI theme
    root.setAttribute('data-theme', dark ? 'dark' : 'light');
    // Background color for instant feedback
    root.style.backgroundColor = dark ? '#111827' : '#f9fafb';
  };

  if (theme === 'dark') {
    apply(true);
  } else if (theme === 'light') {
    apply(false);
  } else {
    // system
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    apply(mq.matches);
    // Remove old listener if any, add new one
    const handler = (e: MediaQueryListEvent) => apply(e.matches);
    mq.addEventListener('change', handler);
  }
}
""")

print("\nСторы исправлены!")
print("\nВыполни:")
print("  cd ~/Antigravity-Manager && git add -A && git commit -m 'Fix stores: correct command names, theme, proxy stop' && git push")
