import React, { useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { BottomNav } from './components/BottomNav';
import { Dashboard } from './pages/Dashboard';
import { Accounts } from './pages/Accounts';
import { Logs } from './pages/Logs';
import { Settings } from './pages/Settings';
import { useConfigStore } from './stores/useConfigStore';
import { useAccountStore } from './stores/useAccountStore';
import { useTranslation } from 'react-i18next';
import { listen } from '@tauri-apps/api/event';
import { isTauri } from './utils/env';
import { request as invoke } from './utils/request';
import { AdminAuthGuard } from './components/common/AdminAuthGuard';
import ThemeManager from './components/common/ThemeManager';
import DebugConsole from './components/debug/DebugConsole';
import { UpdateNotification } from './components/UpdateNotification';

// ─── App Shell with Bottom Navigation ─────────────────────────────────────────
const AppShell: React.FC = () => {
  return (
    <div className="relative min-h-screen bg-gray-50 dark:bg-gray-950">
      <main className="relative">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/accounts" element={<Accounts />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
      <BottomNav />
    </div>
  );
};

// ─── Main App Component ───────────────────────────────────────────────────────
const App: React.FC = () => {
  const { config, loadConfig } = useConfigStore();
  const { fetchCurrentAccount, fetchAccounts } = useAccountStore();
  const { i18n } = useTranslation();
  const [showUpdateNotification, setShowUpdateNotification] = useState(false);

  // Load config on mount
  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  // Sync language from config
  useEffect(() => {
    if (config?.language) {
      i18n.changeLanguage(config.language);
      document.documentElement.dir = config.language === 'ar' ? 'rtl' : 'ltr';
    }
  }, [config?.language, i18n]);

  // Sync system theme (if theme = 'system')
  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => {
      const theme = useConfigStore.getState().config.theme;
      if (theme === 'system') {
        document.documentElement.classList.toggle('dark', mq.matches);
      }
    };
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, []);

  // Listen for Tauri events (tray, account refresh)
  useEffect(() => {
    if (!isTauri()) return;
    const unlistenPromises: Promise<() => void>[] = [];

    unlistenPromises.push(
      listen('tray://account-switched', () => {
        console.log('[App] Tray account switched, refreshing...');
        fetchCurrentAccount();
        fetchAccounts();
      })
    );
    unlistenPromises.push(
      listen('tray://refresh-current', () => {
        console.log('[App] Tray refresh triggered, refreshing...');
        fetchCurrentAccount();
        fetchAccounts();
      })
    );
    unlistenPromises.push(
      listen('accounts://refreshed', () => {
        console.log('[App] Backend triggered quota refresh, syncing UI...');
        fetchCurrentAccount();
        fetchAccounts();
      })
    );

    return () => {
      Promise.all(unlistenPromises).then((unlisteners) => {
        unlisteners.forEach((unlisten) => unlisten());
      });
    };
  }, [fetchCurrentAccount, fetchAccounts]);

  // Update check on startup
  useEffect(() => {
    const checkUpdates = async () => {
      try {
        console.log('[App] Checking if we should check for updates...');
        const shouldCheck = await invoke<boolean>('should_check_updates');
        console.log('[App] Should check updates:', shouldCheck);
        if (shouldCheck) {
          setShowUpdateNotification(true);
          await invoke('update_last_check_time');
          console.log('[App] Update check cycle initiated and last check time updated.');
        }
      } catch (error) {
        console.error('Failed to check update settings:', error);
      }
    };
    const timer = setTimeout(checkUpdates, 2000);
    return () => clearTimeout(timer);
  }, []);

  return (
    <AdminAuthGuard>
      <ThemeManager />
      <DebugConsole />
      {showUpdateNotification && <UpdateNotification onClose={() => setShowUpdateNotification(false)} />}
      <BrowserRouter>
        <AppShell />
      </BrowserRouter>
    </AdminAuthGuard>
  );
};

export default App;
