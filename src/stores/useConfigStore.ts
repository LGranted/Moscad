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
  setTheme: (theme: AppConfig['theme']) => void;
}

export const useConfigStore = create<ConfigState>((set, get) => ({
  config: DEFAULT_CONFIG,
  loading: false,
  error: null,

  fetchConfig: async () => {
    set({ loading: true, error: null });
    try {
      const config = await loadConfig();
      set({ config: { ...DEFAULT_CONFIG, ...config }, loading: false });
      applyTheme(config.theme ?? 'system');
    } catch {
      set({ loading: false });
      applyTheme('system');
    }
  },

  saveConfig: async (partial) => {
    const newConfig = { ...get().config, ...partial };
    set({ config: newConfig });
    try {
      await apiSave(newConfig);
      if (partial.theme !== undefined) applyTheme(partial.theme);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setTheme: (theme) => {
    const newConfig = { ...get().config, theme };
    set({ config: newConfig });
    applyTheme(theme);
    apiSave(newConfig).catch(() => {});
  },
}));

function applyTheme(theme: AppConfig['theme']) {
  const root = document.documentElement;
  if (theme === 'dark') {
    root.classList.add('dark');
  } else if (theme === 'light') {
    root.classList.remove('dark');
  } else {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    root.classList.toggle('dark', prefersDark);
  }
}
