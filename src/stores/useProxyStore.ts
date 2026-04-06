
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
  is_running: false, // will be overwritten by fetchStatus
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
