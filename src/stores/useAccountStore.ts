import { create } from 'zustand';
import type { Account, AccountQuota, AggregatedQuota } from '../types';
import {
  getAccounts,
  deleteAccount as apiDeleteAccount,
  setCurrentAccount as apiSetCurrentAccount,
  clearProxyRateLimit as apiClearRateLimit,
  fetchQuota as apiFetchQuota,
} from '../utils/request';

interface AccountState {
  accounts: Account[];
  quotas: Record<string, AccountQuota>; // key = account_id
  loading: boolean;
  error: string | null;

  // Actions
  fetchAccounts: () => Promise<void>;
  deleteAccount: (id: string) => Promise<void>;
  setCurrentAccount: (id: string) => Promise<void>;
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
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setCurrentAccount: async (id) => {
    try {
      await apiSetCurrentAccount(id);
      await get().fetchAccounts();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearRateLimit: async (id) => {
    try {
      await apiClearRateLimit(id);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  fetchQuotaForAll: async () => {
    const { accounts } = get();
    const results: Record<string, AccountQuota> = {};
    await Promise.allSettled(
      accounts.map(async (acc) => {
        try {
          const q = await apiFetchQuota(acc.id);
          results[acc.id] = q;
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
