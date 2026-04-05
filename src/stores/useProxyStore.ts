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
  api_key: 'test',
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
      // Backend may not implement get_proxy_status yet – use defaults
      set({ status: DEFAULT_STATUS });
    }
  },

  start: async () => {
    set({ loading: true, error: null });
    try {
      await startProxyService();
      await get().fetchStatus();
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  stop: async () => {
    set({ loading: true, error: null });
    try {
      await stopProxyService();
      await get().fetchStatus();
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },
}));
