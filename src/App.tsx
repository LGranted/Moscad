import React, { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { BottomNav } from './components/BottomNav';
import { Dashboard } from './pages/Dashboard';
import { Accounts } from './pages/Accounts';
import { Logs } from './pages/Logs';
import { Settings } from './pages/Settings';
import { useConfigStore } from './stores/useConfigStore';

// ─── App Shell ────────────────────────────────────────────────────────────────

const AppShell: React.FC = () => {
  return (
    <div className="relative min-h-screen bg-gray-50 dark:bg-gray-950">
      <main className="relative">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/accounts" element={<Accounts />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/settings" element={<Settings />} />
          {/* Catch-all: redirect to dashboard */}
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
      <BottomNav />
    </div>
  );
};

// ─── Root ─────────────────────────────────────────────────────────────────────

const App: React.FC = () => {
  const fetchConfig = useConfigStore((s) => s.fetchConfig);

  // Bootstrap – load config (and apply theme) on first render
  useEffect(() => {
    fetchConfig();

    // Sync system theme changes in real time
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

  return (
    <BrowserRouter>
      <AppShell />
    </BrowserRouter>
  );
};

export default App;
