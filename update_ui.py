#!/usr/bin/env python3
import os

BASE = os.path.expanduser("~/Antigravity-Manager")

def write(path, content):
    full = os.path.join(BASE, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, 'w') as f:
        f.write(content)
    print(f"OK {path}")

write("src/index.css", """
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --accent: #16a34a;
    --accent-light: #dcfce7;
  }
  .dark {
    --accent: #2dd4bf;
    --accent-light: #0f2a2a;
  }
  html {
    -webkit-tap-highlight-color: transparent;
    -webkit-font-smoothing: antialiased;
    -webkit-text-size-adjust: 100%;
    text-size-adjust: 100%;
    overflow-x: hidden;
  }
  body {
    overscroll-behavior: none;
    user-select: none;
    -webkit-user-select: none;
  }
  input, textarea, [contenteditable] {
    user-select: text;
    -webkit-user-select: text;
  }
  input[type='number']::-webkit-inner-spin-button,
  input[type='number']::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
}

@layer utilities {
  .pt-safe { padding-top: env(safe-area-inset-top); }
  .pb-safe { padding-bottom: env(safe-area-inset-bottom); }
  .safe-area-pb { padding-bottom: max(0.5rem, env(safe-area-inset-bottom)); }
  .accent-bg { background-color: var(--accent); }
  .accent-text { color: var(--accent); }
  .accent-border { border-color: var(--accent); }
  .accent-light-bg { background-color: var(--accent-light); }
  .animate-spin { animation: spin 1s linear infinite; }
  @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
  .animate-pulse { animation: pulse 2s cubic-bezier(0.4,0,0.6,1) infinite; }
  @keyframes pulse { 0%,100% { opacity:1; } 50% { opacity:0.4; } }
}
""")

write("src/components/BottomNav.tsx", """
import React from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

const HomeIcon = ({ filled }: { filled?: boolean }) => (
  <svg viewBox="0 0 24 24" fill={filled ? "currentColor" : "none"} stroke={filled ? "none" : "currentColor"} strokeWidth={2} className="w-6 h-6">
    {filled
      ? <path d="M10.707 2.293a1 1 0 00-1.414 0l-7 7a1 1 0 001.414 1.414L4 10.414V17a1 1 0 001 1h2a1 1 0 001-1v-2a1 1 0 011-1h2a1 1 0 011 1v2a1 1 0 001 1h2a1 1 0 001-1v-6.586l.293.293a1 1 0 001.414-1.414l-7-7z"/>
      : <path strokeLinecap="round" strokeLinejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
    }
  </svg>
);

const AccountsIcon = ({ filled }: { filled?: boolean }) => (
  <svg viewBox="0 0 24 24" fill={filled ? "currentColor" : "none"} stroke={filled ? "none" : "currentColor"} strokeWidth={2} className="w-6 h-6">
    {filled
      ? <path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z"/>
      : <path strokeLinecap="round" strokeLinejoin="round" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z"/>
    }
  </svg>
);

const LogsIcon = ({ filled }: { filled?: boolean }) => (
  <svg viewBox="0 0 24 24" fill={filled ? "currentColor" : "none"} stroke={filled ? "none" : "currentColor"} strokeWidth={2} className="w-6 h-6">
    {filled
      ? <path fillRule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 6a1 1 0 011-1h6a1 1 0 110 2H7a1 1 0 01-1-1zm1 3a1 1 0 100 2h6a1 1 0 100-2H7z" clipRule="evenodd"/>
      : <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
    }
  </svg>
);

const SettingsIcon = ({ filled }: { filled?: boolean }) => (
  <svg viewBox="0 0 24 24" fill={filled ? "currentColor" : "none"} stroke={filled ? "none" : "currentColor"} strokeWidth={2} className="w-6 h-6">
    {filled
      ? <path fillRule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clipRule="evenodd"/>
      : <><path strokeLinecap="round" strokeLinejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/><path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/></>
    }
  </svg>
);

const TABS = [
  { path: '/',          label: 'Главная',   Icon: HomeIcon },
  { path: '/accounts', label: 'Аккаунты',  Icon: AccountsIcon },
  { path: '/logs',     label: 'Логи',      Icon: LogsIcon },
  { path: '/settings', label: 'Настройки', Icon: SettingsIcon },
];

export const BottomNav: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();
  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 bg-white dark:bg-gray-900 border-t border-gray-100 dark:border-gray-800 safe-area-pb">
      <div className="flex items-stretch">
        {TABS.map(({ path, label, Icon }) => {
          const active = location.pathname === path;
          return (
            <button key={path} onClick={() => navigate(path)}
              className={`relative flex-1 flex flex-col items-center justify-center py-2 gap-0.5 transition-colors ${active ? 'accent-text' : 'text-gray-400 dark:text-gray-500'}`}>
              {active && <span className="absolute top-0 left-1/2 -translate-x-1/2 w-8 h-0.5 rounded-b-full accent-bg"/>}
              <Icon filled={active}/>
              <span className="text-[10px] font-medium">{label}</span>
            </button>
          );
        })}
      </div>
    </nav>
  );
};
""")

write("src/components/AddAccountModal.tsx", """
import React, { useState, useCallback } from 'react';
import { open as openUrl } from '@tauri-apps/plugin-shell';
import { startOAuthFlow, submitOAuthCode, cancelOAuthFlow } from '../utils/request';
import { useAccountStore } from '../stores/useAccountStore';

interface Props { onClose: () => void; }
type Step = 'idle' | 'oauth_started' | 'submitting' | 'done';

export const AddAccountModal: React.FC<Props> = ({ onClose }) => {
  const [step, setStep] = useState<Step>('idle');
  const [oauthUrl, setOauthUrl] = useState('');
  const [oauthState, setOauthState] = useState('');
  const [code, setCode] = useState('');
  const [errorMsg, setErrorMsg] = useState('');
  const [copied, setCopied] = useState(false);
  const fetchAccounts = useAccountStore((s) => s.fetchAccounts);

  const handleStart = useCallback(async () => {
    setErrorMsg('');
    try {
      const result = await startOAuthFlow();
      setOauthUrl(result.url);
      setOauthState(result.state);
      setStep('oauth_started');
      try { await openUrl(result.url); } catch {}
    } catch (e) { setErrorMsg(String(e)); }
  }, []);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(oauthUrl).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [oauthUrl]);

  const handleSubmit = useCallback(async () => {
    if (!code.trim()) return;
    setStep('submitting');
    setErrorMsg('');
    try {
      await submitOAuthCode(code.trim(), oauthState);
      await fetchAccounts();
      setStep('done');
      setTimeout(onClose, 1200);
    } catch (e) {
      setErrorMsg(String(e));
      setStep('oauth_started');
    }
  }, [code, oauthState, fetchAccounts, onClose]);

  const handleCancel = useCallback(async () => {
    try { await cancelOAuthFlow(); } catch {}
    onClose();
  }, [onClose]);

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center bg-black/60">
      <div className="w-full max-w-lg bg-white dark:bg-gray-900 rounded-t-3xl px-5 pt-4 pb-8 shadow-2xl">
        <div className="w-10 h-1 rounded-full bg-gray-200 dark:bg-gray-700 mx-auto mb-5"/>

        {step === 'done' ? (
          <div className="flex flex-col items-center gap-3 py-8">
            <div className="w-14 h-14 rounded-full accent-bg flex items-center justify-center">
              <svg viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth={2.5} className="w-7 h-7">
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7"/>
              </svg>
            </div>
            <p className="text-lg font-bold text-gray-900 dark:text-white">Аккаунт добавлен!</p>
          </div>
        ) : (
          <>
            <h2 className="text-lg font-bold text-gray-900 dark:text-white mb-1">Добавить аккаунт</h2>
            <p className="text-sm text-gray-400 mb-5">Авторизация через Google OAuth 2.0</p>

            <div className={`mb-4 rounded-2xl p-4 ${step === 'idle' ? 'accent-light-bg' : 'bg-gray-50 dark:bg-gray-800'}`}>
              <div className="flex items-center gap-2 mb-2">
                <span className="w-5 h-5 rounded-full accent-bg flex items-center justify-center text-[11px] font-bold text-white shrink-0">1</span>
                <p className="text-sm font-semibold text-gray-800 dark:text-gray-200">Откройте браузер и войдите</p>
              </div>
              <p className="text-xs text-gray-500 dark:text-gray-400 mb-3 pl-7">
                Нажмите кнопку — откроется браузер Google. После входа вы увидите пустую страницу — скопируйте URL из адресной строки и вставьте ниже.
              </p>
              {step === 'idle' ? (
                <button onClick={handleStart}
                  className="w-full py-3 rounded-xl accent-bg text-white font-bold text-sm active:opacity-80">
                  Открыть браузер Google
                </button>
              ) : (
                <div className="flex gap-2">
                  <div className="flex-1 bg-white dark:bg-gray-700 rounded-xl px-3 py-2 text-xs font-mono text-gray-500 truncate border border-gray-100 dark:border-gray-600">
                    {oauthUrl}
                  </div>
                  <button onClick={handleCopy}
                    className={`px-3 py-2 rounded-xl text-xs font-semibold transition-colors ${copied ? 'accent-bg text-white' : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300'}`}>
                    {copied ? 'OK' : 'Копировать'}
                  </button>
                </div>
              )}
            </div>

            {step !== 'idle' && (
              <div className="mb-4 rounded-2xl p-4 bg-gray-50 dark:bg-gray-800">
                <div className="flex items-center gap-2 mb-2">
                  <span className="w-5 h-5 rounded-full accent-bg flex items-center justify-center text-[11px] font-bold text-white shrink-0">2</span>
                  <p className="text-sm font-semibold text-gray-800 dark:text-gray-200">Вставьте URL из браузера</p>
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400 mb-3 pl-7">
                  После авторизации страница будет пустой — скопируйте весь URL из адресной строки и вставьте сюда.
                </p>
                <input value={code} onChange={e => setCode(e.target.value)}
                  placeholder="http://localhost/?code=4/0A..."
                  className="w-full rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 px-4 py-3 text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-green-400 dark:focus:ring-teal-400 mb-3"/>
                <button onClick={handleSubmit} disabled={!code.trim() || step === 'submitting'}
                  className="w-full py-3 rounded-xl accent-bg disabled:opacity-40 text-white font-bold text-sm active:opacity-80">
                  {step === 'submitting' ? 'Проверяем...' : 'Подтвердить'}
                </button>
              </div>
            )}

            {errorMsg && (
              <div className="mb-4 rounded-xl bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 px-4 py-3">
                <p className="text-xs text-red-600 dark:text-red-400">{errorMsg}</p>
              </div>
            )}

            <button onClick={handleCancel} className="w-full py-3 text-gray-400 text-sm font-medium">
              Отмена
            </button>
          </>
        )}
      </div>
    </div>
  );
};
""")

write("src/pages/Dashboard.tsx", """
import React, { useEffect, useCallback, useState } from 'react';
import { useProxyStore } from '../stores/useProxyStore';
import { useAccountStore } from '../stores/useAccountStore';
import { useConfigStore } from '../stores/useConfigStore';
import { clearAllProxyRateLimits, getProxyStats } from '../utils/request';
import type { AggregatedQuota } from '../types';

const MODEL_LABELS: Record<string, string> = {
  'claude-sonnet-4-6': 'Claude Sonnet 4.6',
  'claude-opus-4-6-thinking': 'Claude Opus 4.6 Thinking',
  'gemini-2.5-flash': 'Gemini 2.5 Flash',
  'gemini-2.5-flash-lite': 'Gemini 2.5 Flash Lite',
  'gemini-2.5-flash-thinking': 'Gemini 2.5 Flash Thinking',
  'gemini-2.5-pro': 'Gemini 2.5 Pro',
  'gemini-3-flash': 'Gemini 3 Flash',
  'gemini-3-flash-agent': 'Gemini 3 Flash Agent',
  'gemini-3-pro-high': 'Gemini 3 Pro High',
  'gemini-3-pro-low': 'Gemini 3 Pro Low',
  'gemini-3.1-flash-image': 'Gemini 3.1 Flash Image',
  'gemini-3.1-pro-high': 'Gemini 3.1 Pro High',
  'gemini-3.1-pro-low': 'Gemini 3.1 Pro Low',
};
const modelLabel = (m: string) => MODEL_LABELS[m] ?? m;
const fmt = (n: number) => n >= 1_000_000 ? `${(n/1_000_000).toFixed(1)}M` : n >= 1_000 ? `${(n/1_000).toFixed(0)}K` : String(n);

const CopyIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-4 h-4">
    <rect x="9" y="9" width="13" height="13" rx="2"/>
    <path strokeLinecap="round" strokeLinejoin="round" d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
  </svg>
);

const ProxyCard: React.FC = () => {
  const { status, loading, start, stop, fetchStatus } = useProxyStore();
  const { config } = useConfigStore();
  const proxyUrl = `http://${(status as any).address || config.proxy_host}:${status.port || config.proxy_port}`;
  const apiKey = (status as any).api_key || config.api_key || '—';
  const [copied, setCopied] = useState<'url'|'key'|null>(null);
  const [stats, setStats] = useState<any>({});

  const copy = (text: string, which: 'url'|'key') => {
    navigator.clipboard.writeText(text).catch(() => {});
    setCopied(which);
    setTimeout(() => setCopied(null), 1500);
  };

  useEffect(() => {
    if (!status.is_running) return;
    const load = () => getProxyStats().then((s: any) => setStats(s ?? {})).catch(() => {});
    load();
    const id = setInterval(load, 10_000);
    return () => clearInterval(id);
  }, [status.is_running]);

  return (
    <div className="mx-4 rounded-2xl bg-white dark:bg-gray-800 border border-gray-100 dark:border-gray-700 overflow-hidden shadow-sm">
      <div className={`px-4 py-2 flex items-center justify-between ${status.is_running ? 'accent-bg' : 'bg-gray-200 dark:bg-gray-700'}`}>
        <span className="text-xs font-bold text-white">
          {status.is_running ? '● ПРОКСИ РАБОТАЕТ' : '○ ПРОКСИ ОСТАНОВЛЕН'}
        </span>
        {status.is_running && stats.total_requests != null && (
          <span className="text-xs text-white opacity-80">{fmt(stats.total_requests)} запросов</span>
        )}
      </div>
      <div className="p-4">
        <div className="flex items-center gap-2 mb-2">
          <div className="flex-1 bg-gray-50 dark:bg-gray-700 rounded-xl px-3 py-2.5 flex items-center gap-2 min-w-0">
            <span className="text-[10px] font-bold text-gray-400 uppercase shrink-0">URL</span>
            <span className="text-sm font-mono text-gray-800 dark:text-gray-200 truncate">{proxyUrl}</span>
          </div>
          <button onClick={() => copy(proxyUrl, 'url')}
            className={`p-2.5 rounded-xl shrink-0 transition-colors ${copied==='url' ? 'accent-bg text-white' : 'bg-gray-100 dark:bg-gray-700 text-gray-400'}`}>
            <CopyIcon/>
          </button>
        </div>
        <div className="flex items-center gap-2 mb-4">
          <div className="flex-1 bg-gray-50 dark:bg-gray-700 rounded-xl px-3 py-2.5 flex items-center gap-2 min-w-0">
            <span className="text-[10px] font-bold text-gray-400 uppercase shrink-0">КЛЮЧ</span>
            <span className="text-sm font-mono text-gray-800 dark:text-gray-200 truncate">{apiKey}</span>
          </div>
          <button onClick={() => copy(apiKey, 'key')}
            className={`p-2.5 rounded-xl shrink-0 transition-colors ${copied==='key' ? 'accent-bg text-white' : 'bg-gray-100 dark:bg-gray-700 text-gray-400'}`}>
            <CopyIcon/>
          </button>
        </div>
        <button onClick={status.is_running ? stop : start} disabled={loading}
          className={`w-full py-3 rounded-xl font-bold text-sm disabled:opacity-50 ${
            status.is_running
              ? 'bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 border border-red-200 dark:border-red-800'
              : 'accent-bg text-white'
          }`}>
          {loading ? 'Подождите...' : status.is_running ? 'Остановить прокси' : 'Запустить прокси'}
        </button>
        <button onClick={fetchStatus} className="w-full mt-2 py-1.5 text-xs text-gray-400 dark:text-gray-600">
          Обновить статус
        </button>
      </div>
    </div>
  );
};

const QuotaCard: React.FC<{ quota: AggregatedQuota }> = ({ quota }) => {
  const pct = Math.min(100, (quota.total_requests_remaining / 1000) * 100);
  return (
    <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-4">
      <div className="flex items-start justify-between mb-2">
        <div>
          <p className="text-sm font-semibold text-gray-900 dark:text-white">{modelLabel(quota.model)}</p>
          <p className="text-xs text-gray-400">{quota.accounts_count} акк.</p>
        </div>
        <div className="text-right">
          <p className="text-lg font-bold accent-text">{fmt(quota.total_requests_remaining)}</p>
          <p className="text-[10px] text-gray-400">запросов</p>
        </div>
      </div>
      <div className="h-1.5 rounded-full bg-gray-100 dark:bg-gray-700">
        <div className="h-full rounded-full accent-bg transition-all" style={{ width: `${pct}%` }}/>
      </div>
      {quota.total_tokens_remaining > 0 && (
        <p className="text-[10px] text-gray-400 mt-1 text-right">{fmt(quota.total_tokens_remaining)} токенов</p>
      )}
    </div>
  );
};

export const Dashboard: React.FC = () => {
  const { accounts, fetchAccounts, fetchQuotaForAll, getAggregatedQuotas } = useAccountStore();
  const { fetchStatus, status, start } = useProxyStore();
  const { config, fetchConfig } = useConfigStore();
  const [quotaLoading, setQuotaLoading] = useState(false);
  const [clearingLimits, setClearingLimits] = useState(false);
  const aggregated = getAggregatedQuotas();
  const activeAccounts = accounts.filter(a => !a.disabled);

  const refresh = useCallback(async () => {
    await Promise.all([fetchAccounts(), fetchStatus(), fetchConfig()]);
  }, [fetchAccounts, fetchStatus, fetchConfig]);

  const refreshQuotas = useCallback(async () => {
    setQuotaLoading(true);
    try { await fetchQuotaForAll(); } finally { setQuotaLoading(false); }
  }, [fetchQuotaForAll]);

  useEffect(() => { refresh(); }, []);
  useEffect(() => { if (config.auto_start_proxy && !status.is_running) start(); }, [config.auto_start_proxy]);
  useEffect(() => {
    if (accounts.length === 0) return;
    refreshQuotas();
    const id = setInterval(refreshQuotas, 60_000);
    return () => clearInterval(id);
  }, [accounts.length]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 pb-24">
      <header className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-950 px-4 pt-4 pb-3 flex items-center justify-between border-b border-gray-100 dark:border-gray-800">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">Moscad</h1>
        <button onClick={refresh} className="p-2 rounded-xl active:bg-gray-200 dark:active:bg-gray-700 text-gray-400">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-5 h-5">
            <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
        </button>
      </header>

      <div className="space-y-4 pt-4">
        <ProxyCard/>

        <div className="mx-4 flex gap-3">
          {[
            { label: 'аккаунтов', value: accounts.length, cls: 'text-gray-900 dark:text-white' },
            { label: 'активных', value: activeAccounts.length, cls: 'accent-text' },
            { label: 'моделей', value: aggregated.length, cls: 'text-blue-600 dark:text-blue-400' },
          ].map(({ label, value, cls }) => (
            <div key={label} className="flex-1 bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-3 text-center">
              <p className={`text-2xl font-bold ${cls}`}>{value}</p>
              <p className="text-[11px] text-gray-400 mt-0.5">{label}</p>
            </div>
          ))}
        </div>

        <div className="mx-4">
          <div className="flex items-center justify-between mb-3">
            <h2 className="font-semibold text-gray-900 dark:text-white">Лимиты по моделям</h2>
            <button onClick={refreshQuotas} disabled={quotaLoading} className="text-xs accent-text font-semibold disabled:opacity-40">
              {quotaLoading ? 'Загрузка...' : 'Обновить'}
            </button>
          </div>

          {aggregated.length === 0 && !quotaLoading && (
            <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-8 text-center">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-10 h-10 text-gray-200 dark:text-gray-700 mx-auto mb-3">
                <path strokeLinecap="round" strokeLinejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
              </svg>
              <p className="text-sm text-gray-400">
                {accounts.length === 0 ? 'Добавьте аккаунты для просмотра лимитов' : 'Нажмите «Обновить» для загрузки квот'}
              </p>
            </div>
          )}

          {quotaLoading && aggregated.length === 0 && (
            <div className="space-y-3">
              {[1,2,3].map(i => <div key={i} className="bg-white dark:bg-gray-800 rounded-2xl h-20 animate-pulse border border-gray-100 dark:border-gray-700"/>)}
            </div>
          )}

          <div className="space-y-3">
            {aggregated.map(q => <QuotaCard key={q.model} quota={q}/>)}
          </div>
        </div>

        {accounts.length > 0 && (
          <div className="mx-4 pb-2">
            <button onClick={async () => { setClearingLimits(true); try { await clearAllProxyRateLimits(); } finally { setClearingLimits(false); } }}
              disabled={clearingLimits}
              className="w-full py-3 rounded-2xl border border-orange-200 dark:border-orange-800 text-orange-600 dark:text-orange-400 text-sm font-semibold active:bg-orange-50 disabled:opacity-50">
              {clearingLimits ? 'Очищаем...' : '⚡ Сбросить все rate limits'}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
""")

write("src/pages/Accounts.tsx", """
import React, { useEffect, useState, useCallback } from 'react';
import { useAccountStore } from '../stores/useAccountStore';
import { useConfigStore } from '../stores/useConfigStore';
import { AddAccountModal } from '../components/AddAccountModal';
import { warmUpAccount } from '../utils/request';
import type { Account, AccountQuota } from '../types';

const COLORS = ['bg-emerald-500','bg-teal-500','bg-cyan-500','bg-blue-500','bg-violet-500','bg-purple-500','bg-pink-500','bg-rose-500'];
const avatarColor = (email: string) => COLORS[email.charCodeAt(0) % COLORS.length];

interface CardProps {
  account: Account;
  quota?: AccountQuota;
  onDelete: () => void;
  onSwitch: () => void;
  onClearLimit: () => void;
}

const AccountCard: React.FC<CardProps> = ({ account, quota, onDelete, onSwitch, onClearLimit }) => {
  const [expanded, setExpanded] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [warming, setWarming] = useState(false);
  const isActive = !account.disabled;

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-2xl border overflow-hidden transition-colors ${
      account.is_current ? 'border-green-300 dark:border-teal-600' : 'border-gray-100 dark:border-gray-700'
    }`}>
      {account.is_current && <div className="h-0.5 accent-bg"/>}
      <button className="w-full flex items-center gap-3 p-4 text-left active:bg-gray-50 dark:active:bg-gray-700/30"
        onClick={() => setExpanded(e => !e)}>
        <div className={`w-10 h-10 rounded-full flex items-center justify-center text-white text-sm font-bold shrink-0 ${avatarColor(account.email)}`}>
          {account.email?.[0]?.toUpperCase() ?? '?'}
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-semibold text-gray-900 dark:text-white truncate">{account.email}</p>
          <div className="flex items-center gap-2 mt-0.5">
            {account.is_current && <span className="text-[10px] font-bold accent-text uppercase">● Текущий</span>}
            <span className={`text-xs ${isActive ? 'accent-text' : 'text-red-400'}`}>
              {isActive ? 'Активен' : 'Отключён'}
            </span>
          </div>
        </div>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}
          className={`w-4 h-4 text-gray-400 shrink-0 transition-transform ${expanded ? 'rotate-180' : ''}`}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7"/>
        </svg>
      </button>

      {expanded && (
        <div className="px-4 pb-4 space-y-3 border-t border-gray-100 dark:border-gray-700 pt-3">
          {quota?.error ? (
            <p className="text-xs text-red-500">{quota.error}</p>
          ) : quota?.quotas && quota.quotas.length > 0 ? (
            <div className="space-y-1.5">
              {quota.quotas.slice(0, 4).map(mq => (
                <div key={mq.model} className="flex items-center justify-between">
                  <span className="text-xs text-gray-500 dark:text-gray-400 truncate max-w-[60%]">{mq.model}</span>
                  <span className="text-xs font-semibold accent-text">{mq.requests_remaining ?? '—'}</span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-xs text-gray-400">Квоты не загружены</p>
          )}

          <div className="flex gap-2 flex-wrap">
            {!account.is_current && (
              <button onClick={onSwitch}
                className="flex-1 py-2 rounded-xl accent-light-bg accent-text text-xs font-semibold active:opacity-70">
                Сделать текущим
              </button>
            )}
            <button onClick={async () => { setWarming(true); try { await warmUpAccount(account.id); } finally { setWarming(false); } }}
              disabled={warming}
              className="flex-1 py-2 rounded-xl bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 text-xs font-semibold active:opacity-70 disabled:opacity-40">
              {warming ? 'Прогрев...' : '⚡ Прогреть'}
            </button>
            <button onClick={onClearLimit}
              className="flex-1 py-2 rounded-xl bg-orange-50 dark:bg-orange-900/20 text-orange-500 text-xs font-semibold active:opacity-70">
              Сброс лимита
            </button>
          </div>

          {confirmDelete ? (
            <div className="flex gap-2">
              <button onClick={() => setConfirmDelete(false)}
                className="flex-1 py-2 rounded-xl bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 text-xs font-semibold">
                Отмена
              </button>
              <button onClick={onDelete}
                className="flex-1 py-2 rounded-xl bg-red-500 text-white text-xs font-bold active:bg-red-600">
                Удалить
              </button>
            </div>
          ) : (
            <button onClick={() => setConfirmDelete(true)}
              className="w-full py-2 rounded-xl bg-red-50 dark:bg-red-900/20 text-red-500 text-xs font-semibold">
              Удалить аккаунт
            </button>
          )}
        </div>
      )}
    </div>
  );
};

export const Accounts: React.FC = () => {
  const { accounts, quotas, loading, fetchAccounts, deleteAccount, switchAccount, clearRateLimit, fetchQuotaForAll } = useAccountStore();
  const { config, saveConfig } = useConfigStore();
  const [showAdd, setShowAdd] = useState(false);
  const [quotaLoading, setQuotaLoading] = useState(false);

  const refresh = useCallback(async () => { await fetchAccounts(); }, [fetchAccounts]);
  const refreshQuotas = useCallback(async () => {
    setQuotaLoading(true);
    try { await fetchQuotaForAll(); } finally { setQuotaLoading(false); }
  }, [fetchQuotaForAll]);

  useEffect(() => { refresh(); }, []);
  useEffect(() => { if (accounts.length > 0) refreshQuotas(); }, [accounts.length]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 pb-24">
      <header className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-950 px-4 pt-4 pb-3 flex items-center justify-between border-b border-gray-100 dark:border-gray-800">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">Аккаунты</h1>
        <div className="flex items-center gap-2">
          <button onClick={refresh} className="p-2 rounded-xl text-gray-400 active:bg-gray-200 dark:active:bg-gray-700">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className={`w-5 h-5 ${loading ? 'animate-spin' : ''}`}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
            </svg>
          </button>
          <button onClick={() => setShowAdd(true)}
            className="flex items-center gap-1.5 px-4 py-2 rounded-xl accent-bg text-white text-sm font-bold active:opacity-80">
            <span>+</span><span>Добавить</span>
          </button>
        </div>
      </header>

      <div className="px-4 pt-4 space-y-4">
        <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-4 flex items-center justify-between">
          <div>
            <p className="text-sm font-semibold text-gray-900 dark:text-white">Ротация аккаунтов</p>
            <p className="text-xs text-gray-400 mt-0.5">Переключать при исчерпании лимитов</p>
          </div>
          <button onClick={() => saveConfig({ rotation_enabled: !config.rotation_enabled })}
            className={`relative w-12 h-6 rounded-full transition-colors ${config.rotation_enabled ? 'accent-bg' : 'bg-gray-300 dark:bg-gray-600'}`}>
            <span className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform ${config.rotation_enabled ? 'translate-x-6' : ''}`}/>
          </button>
        </div>

        {accounts.length === 0 && !loading && (
          <div className="text-center py-14">
            <div className="w-16 h-16 rounded-full bg-gray-100 dark:bg-gray-800 flex items-center justify-center mx-auto mb-4">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-8 h-8 text-gray-300 dark:text-gray-600">
                <path strokeLinecap="round" strokeLinejoin="round" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z"/>
              </svg>
            </div>
            <p className="font-semibold text-gray-500 dark:text-gray-400">Нет аккаунтов</p>
            <p className="text-sm text-gray-400 dark:text-gray-600 mt-1">Нажмите «Добавить», чтобы начать</p>
          </div>
        )}

        <div className="space-y-3">
          {accounts.map(acc => (
            <AccountCard key={acc.id} account={acc} quota={quotas[acc.id]}
              onDelete={() => deleteAccount(acc.id)}
              onSwitch={() => switchAccount(acc.id)}
              onClearLimit={() => clearRateLimit(acc.id)}/>
          ))}
        </div>

        {accounts.length > 0 && (
          <button onClick={refreshQuotas} disabled={quotaLoading}
            className="w-full py-3 rounded-2xl border border-gray-200 dark:border-gray-700 text-gray-500 text-sm disabled:opacity-40">
            {quotaLoading ? 'Обновление квот...' : 'Обновить все квоты'}
          </button>
        )}
      </div>

      {showAdd && <AddAccountModal onClose={() => { setShowAdd(false); refresh(); }}/>}
    </div>
  );
};
""")

write("src/pages/Logs.tsx", """
import React, { useEffect, useState, useCallback, useRef } from 'react';
import { getLogsFiltered, getLogsCount, clearProxyLogs } from '../utils/request';
import type { LogEntry } from '../types';

const METHOD_COLORS: Record<string, string> = {
  GET:    'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400',
  POST:   'bg-emerald-100 dark:bg-emerald-900/40 text-emerald-700 dark:text-emerald-400',
  PUT:    'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400',
  DELETE: 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-400',
  PATCH:  'bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-400',
};

const statusColor = (code: number) => {
  if (code >= 500) return 'text-red-600 dark:text-red-400';
  if (code >= 400) return 'text-orange-500 dark:text-orange-400';
  if (code >= 200) return 'accent-text';
  return 'text-gray-400';
};

const formatTime = (ts: string) => {
  try { return new Date(ts).toLocaleTimeString('ru-RU', { hour:'2-digit', minute:'2-digit', second:'2-digit' }); }
  catch { return ts; }
};

const LogRow: React.FC<{ entry: LogEntry }> = ({ entry }) => {
  const [expanded, setExpanded] = useState(false);
  const isError = entry.status_code >= 400 || !!entry.error;
  return (
    <div className={`rounded-2xl border ${isError
      ? 'bg-red-50 dark:bg-red-900/10 border-red-100 dark:border-red-900'
      : 'bg-white dark:bg-gray-800 border-gray-100 dark:border-gray-700'}`}>
      <button className="w-full p-3 text-left active:opacity-70" onClick={() => setExpanded(e => !e)}>
        <div className="flex items-center gap-2 mb-1">
          <span className={`text-[10px] font-bold px-1.5 py-0.5 rounded-md ${METHOD_COLORS[entry.method] ?? 'bg-gray-100 dark:bg-gray-700 text-gray-500'}`}>
            {entry.method}
          </span>
          <span className={`text-xs font-bold ${statusColor(entry.status_code)}`}>{entry.status_code}</span>
          {entry.duration_ms != null && <span className="text-[10px] text-gray-400">{entry.duration_ms}мс</span>}
          <span className="ml-auto text-[10px] text-gray-400">{formatTime(entry.timestamp)}</span>
        </div>
        <p className="text-xs text-gray-700 dark:text-gray-300 font-mono leading-tight truncate">{entry.url}</p>
        {entry.account_email && <p className="text-[10px] text-gray-400 mt-0.5">{entry.account_email}</p>}
        {entry.error && <p className="text-xs text-red-500 mt-1 truncate">{entry.error}</p>}
      </button>
      {expanded && (
        <div className="px-3 pb-3 border-t border-gray-100 dark:border-gray-700 pt-2 space-y-2">
          {entry.model && <p className="text-xs text-gray-500"><span className="font-semibold">Модель:</span> {entry.model}</p>}
          <p className="text-xs text-gray-500 font-mono break-all">{entry.url}</p>
          {entry.error && (
            <div className="rounded-xl bg-red-100 dark:bg-red-900/30 p-2">
              <p className="text-xs text-red-700 dark:text-red-300 font-mono break-all">{entry.error}</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export const Logs: React.FC = () => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [errorsOnly, setErrorsOnly] = useState(false);
  const [search, setSearch] = useState('');
  const [totalCount, setTotalCount] = useState(0);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchLogs = useCallback(async () => {
    setLoading(true);
    try {
      const [entries, count] = await Promise.all([
        getLogsFiltered(errorsOnly, search, 200),
        getLogsCount(errorsOnly, search),
      ]);
      setLogs(entries);
      setTotalCount(count);
    } catch {} finally { setLoading(false); }
  }, [errorsOnly, search]);

  const handleClear = async () => {
    setClearing(true);
    try { await clearProxyLogs(); setLogs([]); setTotalCount(0); }
    finally { setClearing(false); }
  };

  useEffect(() => { fetchLogs(); }, [errorsOnly, search]);
  useEffect(() => {
    if (intervalRef.current) clearInterval(intervalRef.current);
    intervalRef.current = setInterval(fetchLogs, 5_000);
    return () => { if (intervalRef.current) clearInterval(intervalRef.current); };
  }, [fetchLogs]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 pb-24">
      <header className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-950 px-4 pt-4 pb-3 border-b border-gray-100 dark:border-gray-800">
        <div className="flex items-center justify-between mb-3">
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">
            Логи
            {totalCount > 0 && <span className="ml-2 text-sm font-normal text-gray-400">({totalCount})</span>}
          </h1>
          <div className="flex items-center gap-1">
            {loading && (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-4 h-4 accent-text animate-spin mr-1">
                <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
              </svg>
            )}
            <button onClick={handleClear} disabled={clearing || logs.length === 0}
              className="p-2 rounded-xl text-gray-400 active:bg-gray-200 dark:active:bg-gray-700 disabled:opacity-30">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-5 h-5">
                <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
              </svg>
            </button>
            <button onClick={fetchLogs} className="p-2 rounded-xl text-gray-400 active:bg-gray-200 dark:active:bg-gray-700">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-5 h-5">
                <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
              </svg>
            </button>
          </div>
        </div>
        <div className="flex gap-2">
          <div className="flex-1 flex items-center gap-2 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 px-3 py-2">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-4 h-4 text-gray-400 shrink-0">
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
            <input value={search} onChange={e => setSearch(e.target.value)} placeholder="Поиск..."
              className="flex-1 bg-transparent text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none"/>
            {search && <button onClick={() => setSearch('')} className="text-gray-400 text-sm">x</button>}
          </div>
          <button onClick={() => setErrorsOnly(e => !e)}
            className={`px-3 py-2 rounded-xl text-xs font-bold transition-colors ${errorsOnly ? 'bg-red-500 text-white' : 'bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-gray-500'}`}>
            Ошибки
          </button>
        </div>
      </header>

      <div className="px-4 pt-4 space-y-2">
        {logs.length === 0 && !loading && (
          <div className="text-center py-14">
            <div className="w-16 h-16 rounded-full bg-gray-100 dark:bg-gray-800 flex items-center justify-center mx-auto mb-4">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} className="w-8 h-8 text-gray-300 dark:text-gray-600">
                <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
              </svg>
            </div>
            <p className="font-semibold text-gray-400 dark:text-gray-500">
              {errorsOnly || search ? 'Нет совпадений' : 'Логи пусты'}
            </p>
            <p className="text-sm text-gray-300 dark:text-gray-600 mt-1">Логи появятся после запуска прокси</p>
          </div>
        )}
        {logs.map(entry => <LogRow key={entry.id} entry={entry}/>)}
      </div>
    </div>
  );
};
""")

write("src/pages/Settings.tsx", """
import React, { useEffect, useState } from 'react';
import { useConfigStore } from '../stores/useConfigStore';
import { clearAllProxyRateLimits, checkProxyHealth } from '../utils/request';
import type { AppConfig } from '../types';

const ToggleRow: React.FC<{ label: string; desc?: string; value: boolean; onChange: (v: boolean) => void }> = ({ label, desc, value, onChange }) => (
  <div className="flex items-center justify-between py-3.5 border-b border-gray-100 dark:border-gray-700 last:border-0">
    <div className="flex-1 mr-3">
      <p className="text-sm font-medium text-gray-900 dark:text-white">{label}</p>
      {desc && <p className="text-xs text-gray-400 mt-0.5">{desc}</p>}
    </div>
    <button onClick={() => onChange(!value)}
      className={`relative w-12 h-6 rounded-full transition-colors shrink-0 ${value ? 'accent-bg' : 'bg-gray-300 dark:bg-gray-600'}`}>
      <span className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform ${value ? 'translate-x-6' : ''}`}/>
    </button>
  </div>
);

const SelectRow: React.FC<{ label: string; value: string; options: {value:string;label:string}[]; onChange: (v:string) => void }> = ({ label, value, options, onChange }) => (
  <div className="flex items-center justify-between py-3.5 border-b border-gray-100 dark:border-gray-700 last:border-0">
    <p className="text-sm font-medium text-gray-900 dark:text-white">{label}</p>
    <select value={value} onChange={e => onChange(e.target.value)}
      className="bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white text-sm rounded-xl px-3 py-1.5 focus:outline-none">
      {options.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
    </select>
  </div>
);

const NumberRow: React.FC<{ label: string; desc?: string; value: number; min?: number; max?: number; step?: number; onChange: (v:number) => void }> = ({ label, desc, value, min, max, step=1, onChange }) => {
  const [local, setLocal] = useState(String(value));
  useEffect(() => setLocal(String(value)), [value]);
  return (
    <div className="flex items-center justify-between py-3.5 border-b border-gray-100 dark:border-gray-700 last:border-0">
      <div className="flex-1 mr-3">
        <p className="text-sm font-medium text-gray-900 dark:text-white">{label}</p>
        {desc && <p className="text-xs text-gray-400 mt-0.5">{desc}</p>}
      </div>
      <input type="number" value={local} min={min} max={max} step={step}
        onChange={e => setLocal(e.target.value)}
        onBlur={() => { const n = Number(local); if (!isNaN(n)) onChange(n); }}
        className="w-24 bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white text-sm rounded-xl px-3 py-1.5 focus:outline-none text-right"/>
    </div>
  );
};

const ReadonlyRow: React.FC<{ label: string; value: string }> = ({ label, value }) => {
  const [copied, setCopied] = useState(false);
  return (
    <div className="flex items-center justify-between py-3.5 border-b border-gray-100 dark:border-gray-700 last:border-0">
      <p className="text-sm font-medium text-gray-900 dark:text-white">{label}</p>
      <div className="flex items-center gap-2">
        <span className="text-xs font-mono text-gray-400 max-w-[100px] truncate">{value || 'нет'}</span>
        <button onClick={() => { navigator.clipboard.writeText(value).catch(() => {}); setCopied(true); setTimeout(() => setCopied(false), 1500); }}
          className={`text-xs px-2 py-1 rounded-lg transition-colors ${copied ? 'accent-bg text-white' : 'bg-gray-100 dark:bg-gray-700 text-gray-500'}`}>
          {copied ? 'OK' : 'Копировать'}
        </button>
      </div>
    </div>
  );
};

const Section: React.FC<{ title: string; children: React.ReactNode }> = ({ title, children }) => (
  <div className="mx-4 mb-4">
    <h2 className="text-xs font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider px-1 mb-2">{title}</h2>
    <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 px-4">
      {children}
    </div>
  </div>
);

export const Settings: React.FC = () => {
  const { config, saveConfig, fetchConfig } = useConfigStore();
  const [saving, setSaving] = useState(false);
  const [savedMsg, setSavedMsg] = useState(false);
  const [clearingLimits, setClearingLimits] = useState(false);
  const [healthStatus, setHealthStatus] = useState<boolean | null>(null);
  const [checkingHealth, setCheckingHealth] = useState(false);

  useEffect(() => { fetchConfig(); }, []);

  const update = async (partial: Partial<AppConfig>) => {
    setSaving(true);
    await saveConfig(partial);
    setSaving(false);
    setSavedMsg(true);
    setTimeout(() => setSavedMsg(false), 1500);
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 pb-24">
      <header className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-950 px-4 pt-4 pb-3 flex items-center justify-between border-b border-gray-100 dark:border-gray-800">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">Настройки</h1>
        {(saving || savedMsg) && (
          <span className={`text-xs font-semibold ${savedMsg ? 'accent-text' : 'text-gray-400'}`}>
            {saving ? 'Сохранение...' : 'Сохранено'}
          </span>
        )}
      </header>

      <div className="pt-4">
        <Section title="Внешний вид">
          <SelectRow label="Тема" value={config.theme}
            options={[{value:'system',label:'Системная'},{value:'light',label:'Светлая'},{value:'dark',label:'Тёмная'}]}
            onChange={v => update({ theme: v as AppConfig['theme'] })}/>
          <SelectRow label="Язык" value={config.language}
            options={[{value:'ru',label:'Русский'},{value:'en',label:'English'}]}
            onChange={v => update({ language: v })}/>
        </Section>

        <Section title="Прокси">
          <ToggleRow label="Автозапуск при старте" desc="Запускать прокси при открытии"
            value={config.auto_start_proxy} onChange={v => update({ auto_start_proxy: v })}/>
          <NumberRow label="Порт" value={config.proxy_port} min={1024} max={65535} onChange={v => update({ proxy_port: v })}/>
          <NumberRow label="Таймаут (мс)" value={config.request_timeout_ms} min={1000} step={1000} onChange={v => update({ request_timeout_ms: v })}/>
          <ReadonlyRow label="API ключ" value={config.api_key}/>
          <div className="py-3.5">
            <button onClick={async () => {
              setCheckingHealth(true); setHealthStatus(null);
              try { setHealthStatus(await checkProxyHealth()); } catch { setHealthStatus(false); }
              finally { setCheckingHealth(false); }
            }} disabled={checkingHealth}
              className="w-full py-2.5 rounded-xl border border-gray-200 dark:border-gray-700 text-sm font-medium text-gray-600 dark:text-gray-300 active:bg-gray-100 disabled:opacity-40">
              {checkingHealth ? 'Проверяем...' : 'Проверить здоровье прокси'}
            </button>
            {healthStatus !== null && (
              <p className={`text-xs text-center mt-2 font-semibold ${healthStatus ? 'accent-text' : 'text-red-500'}`}>
                {healthStatus ? 'Прокси работает нормально' : 'Прокси недоступен'}
              </p>
            )}
          </div>
        </Section>

        <Section title="Повторные запросы">
          <ToggleRow label="Автоматические повторы" desc="Повторять при ошибке 429"
            value={config.retry_enabled} onChange={v => update({ retry_enabled: v })}/>
          {config.retry_enabled && (
            <>
              <NumberRow label="Макс. повторов" value={config.retry_limit} min={0} max={20} onChange={v => update({ retry_limit: v })}/>
              <NumberRow label="Задержка (мс)" value={config.retry_base_delay_ms} min={100} step={100} onChange={v => update({ retry_base_delay_ms: v })}/>
            </>
          )}
          <div className="py-3.5">
            <button onClick={async () => { setClearingLimits(true); try { await clearAllProxyRateLimits(); } finally { setClearingLimits(false); } }}
              disabled={clearingLimits}
              className="w-full py-2.5 rounded-xl bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 text-orange-600 dark:text-orange-400 text-sm font-semibold disabled:opacity-40">
              {clearingLimits ? 'Очищаем...' : 'Очистить все rate limits'}
            </button>
          </div>
        </Section>

        <Section title="Логи">
          <ToggleRow label="Запись логов" value={config.log_enabled} onChange={v => update({ log_enabled: v })}/>
          {config.log_enabled && (
            <SelectRow label="Уровень" value={config.log_level}
              options={[{value:'debug',label:'Debug'},{value:'info',label:'Info'},{value:'warn',label:'Warn'},{value:'error',label:'Error'}]}
              onChange={v => update({ log_level: v as AppConfig['log_level'] })}/>
          )}
        </Section>

        <div className="text-center py-6">
          <p className="text-xs text-gray-300 dark:text-gray-700">Moscad · Google Antigravity Proxy</p>
        </div>
      </div>
    </div>
  );
};
""")

print("\nВсе файлы записаны!")
print("\nВыполни в Termux:")
print("  cd ~/Antigravity-Manager && git add -A && git commit -m 'UI overhaul: green/teal theme, SVG icons, fixed UX' && git push")
