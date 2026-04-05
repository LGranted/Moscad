
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
