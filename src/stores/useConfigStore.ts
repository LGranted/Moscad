
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

let _systemThemeHandler: ((e: MediaQueryListEvent) => void) | null = null;

function applyTheme(theme: AppConfig['theme']) {
  const root = document.documentElement;
  const mq = window.matchMedia('(prefers-color-scheme: dark)');

  // Remove previous system listener if any
  if (_systemThemeHandler) {
    mq.removeEventListener('change', _systemThemeHandler);
    _systemThemeHandler = null;
  }

  const apply = (dark: boolean) => {
    root.classList.toggle('dark', dark);
    root.setAttribute('data-theme', dark ? 'dark' : 'light');
    root.style.backgroundColor = dark ? '#111827' : '#f9fafb';
  };

  // Persist preference
  localStorage.setItem('theme', theme);

  if (theme === 'dark') {
    apply(true);
  } else if (theme === 'light') {
    apply(false);
  } else {
    apply(mq.matches);
    _systemThemeHandler = (e: MediaQueryListEvent) => apply(e.matches);
    mq.addEventListener('change', _systemThemeHandler);
  }
}

// Apply theme on boot from localStorage before config loads
(function initTheme() {
  const saved = localStorage.getItem('theme') as AppConfig['theme'] | null;
  if (saved) applyTheme(saved);
})();
