import React, { useEffect, useCallback, useState } from 'react';
import { useProxyStore } from '../stores/useProxyStore';
import { useAccountStore } from '../stores/useAccountStore';
import { useConfigStore } from '../stores/useConfigStore';
import { clearAllProxyRateLimits } from '../utils/request';
import type { AggregatedQuota } from '../types';

// ─── Subcomponents ───────────────────────────────────────────────────────────

const ProxyCard: React.FC = () => {
  const { status, loading, start, stop, fetchStatus } = useProxyStore();
  const { config } = useConfigStore();

  const proxyUrl = `http://${status.address || config.proxy_host}:${status.port || config.proxy_port}`;
  const apiKey = status.api_key || config.api_key;

  const [copied, setCopied] = useState<'url' | 'key' | null>(null);
  const copy = (text: string, which: 'url' | 'key') => {
    navigator.clipboard.writeText(text).catch(() => {});
    setCopied(which);
    setTimeout(() => setCopied(null), 1500);
  };

  return (
    <div className="mx-4 rounded-2xl bg-white dark:bg-gray-800 shadow-sm border border-gray-100 dark:border-gray-700 p-4">
      <div className="flex items-center justify-between mb-3">
        <h2 className="font-semibold text-gray-900 dark:text-white">Прокси</h2>
        <span className={`px-2.5 py-1 rounded-full text-xs font-semibold ${
          status.is_running
            ? 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400'
            : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400'
        }`}>
          {status.is_running ? '● Работает' : '○ Остановлен'}
        </span>
      </div>

      {/* URL */}
      <div className="flex items-center gap-2 mb-2">
        <div className="flex-1 bg-gray-50 dark:bg-gray-700 rounded-xl px-3 py-2 flex items-center gap-2">
          <span className="text-xs text-gray-500 dark:text-gray-400 shrink-0">URL</span>
          <span className="text-sm font-mono text-gray-800 dark:text-gray-200 truncate">{proxyUrl}</span>
        </div>
        <button
          onClick={() => copy(proxyUrl, 'url')}
          className="p-2 rounded-xl bg-gray-100 dark:bg-gray-700 active:opacity-60"
        >
          <span className="text-xs">{copied === 'url' ? '✓' : '⎘'}</span>
        </button>
      </div>

      {/* API Key */}
      <div className="flex items-center gap-2 mb-4">
        <div className="flex-1 bg-gray-50 dark:bg-gray-700 rounded-xl px-3 py-2 flex items-center gap-2">
          <span className="text-xs text-gray-500 dark:text-gray-400 shrink-0">Ключ</span>
          <span className="text-sm font-mono text-gray-800 dark:text-gray-200">{apiKey}</span>
        </div>
        <button
          onClick={() => copy(apiKey, 'key')}
          className="p-2 rounded-xl bg-gray-100 dark:bg-gray-700 active:opacity-60"
        >
          <span className="text-xs">{copied === 'key' ? '✓' : '⎘'}</span>
        </button>
      </div>

      {/* Toggle button */}
      <button
        onClick={status.is_running ? stop : start}
        disabled={loading}
        className={`w-full py-3 rounded-xl font-semibold text-sm transition-colors
          ${status.is_running
            ? 'bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 active:bg-red-200'
            : 'bg-blue-600 text-white active:bg-blue-700'}
          disabled:opacity-50`}
      >
        {loading ? 'Подождите...' : status.is_running ? 'Остановить прокси' : 'Запустить прокси'}
      </button>

      <button
        onClick={fetchStatus}
        className="w-full mt-2 py-2 text-xs text-gray-400 dark:text-gray-500 active:text-gray-600"
      >
        Обновить статус
      </button>
    </div>
  );
};

const MODEL_LABELS: Record<string, string> = {
  'claude-opus-4': 'Claude Opus 4',
  'claude-opus-4-5': 'Claude Opus 4.5',
  'claude-opus-4-6': 'Claude Opus 4.6',
  'claude-sonnet-4': 'Claude Sonnet 4',
  'claude-sonnet-4-5': 'Claude Sonnet 4.5',
  'claude-sonnet-4-6': 'Claude Sonnet 4.6',
  'claude-haiku-4-5': 'Claude Haiku 4.5',
  'gemini-2.5-pro': 'Gemini 2.5 Pro',
  'gemini-2.0-flash': 'Gemini 2.0 Flash',
  'gemini-1.5-pro': 'Gemini 1.5 Pro',
  'gemini-1.5-flash': 'Gemini 1.5 Flash',
};

function modelLabel(model: string): string {
  return MODEL_LABELS[model] ?? model;
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(0)}K`;
  return String(n);
}

const QuotaCard: React.FC<{ quota: AggregatedQuota }> = ({ quota }) => {
  const reqPct = quota.total_requests_remaining > 0
    ? Math.min(100, (quota.total_requests_remaining / 1000) * 100)
    : 0;

  return (
    <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-4">
      <div className="flex items-start justify-between mb-3">
        <div>
          <p className="text-sm font-semibold text-gray-900 dark:text-white">{modelLabel(quota.model)}</p>
          <p className="text-xs text-gray-400 dark:text-gray-500">{quota.accounts_count} аккаунт(ов)</p>
        </div>
        <div className="text-right">
          <p className="text-lg font-bold text-blue-600 dark:text-blue-400">
            {formatNumber(quota.total_requests_remaining)}
          </p>
          <p className="text-xs text-gray-400">запросов</p>
        </div>
      </div>
      <div className="h-1.5 rounded-full bg-gray-100 dark:bg-gray-700">
        <div
          className="h-full rounded-full bg-blue-500 dark:bg-blue-400 transition-all"
          style={{ width: `${reqPct}%` }}
        />
      </div>
      {quota.total_tokens_remaining > 0 && (
        <p className="text-xs text-gray-400 dark:text-gray-500 mt-1 text-right">
          {formatNumber(quota.total_tokens_remaining)} токенов
        </p>
      )}
    </div>
  );
};

// ─── Main Dashboard ──────────────────────────────────────────────────────────

export const Dashboard: React.FC = () => {
  const { accounts, fetchAccounts, fetchQuotaForAll, getAggregatedQuotas } = useAccountStore();
  const { fetchStatus, status } = useProxyStore();
  const { config, fetchConfig } = useConfigStore();

  const [quotaLoading, setQuotaLoading] = useState(false);
  const [clearingLimits, setClearingLimits] = useState(false);
  const aggregated = getAggregatedQuotas();
  const validAccounts = accounts.filter((a) => a.is_valid);

  const refresh = useCallback(async () => {
    await Promise.all([fetchAccounts(), fetchStatus(), fetchConfig()]);
  }, [fetchAccounts, fetchStatus, fetchConfig]);

  const refreshQuotas = useCallback(async () => {
    setQuotaLoading(true);
    try {
      await fetchQuotaForAll();
    } finally {
      setQuotaLoading(false);
    }
  }, [fetchQuotaForAll]);

  const handleClearAll = useCallback(async () => {
    setClearingLimits(true);
    try { await clearAllProxyRateLimits(); } finally { setClearingLimits(false); }
  }, []);

  // Initial load
  useEffect(() => {
    refresh();
  }, []);

  // Auto-start proxy if enabled in config
  const { start } = useProxyStore();
  useEffect(() => {
    if (config.auto_start_proxy && !status.is_running) {
      start();
    }
  }, [config.auto_start_proxy]);

  // Periodic quota refresh (every 60s)
  useEffect(() => {
    if (accounts.length === 0) return;
    refreshQuotas();
    const id = setInterval(refreshQuotas, 60_000);
    return () => clearInterval(id);
  }, [accounts.length]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 pb-24">
      {/* Header */}
      <header className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-950 px-4 pt-safe pt-4 pb-3 flex items-center justify-between border-b border-gray-100 dark:border-gray-800">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">Moscad</h1>
        <button
          onClick={refresh}
          className="p-2 rounded-xl active:bg-gray-200 dark:active:bg-gray-700"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-5 h-5 text-gray-500 dark:text-gray-400">
            <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      </header>

      <div className="space-y-4 pt-4">
        {/* Proxy Card */}
        <ProxyCard />

        {/* Account stats */}
        <div className="mx-4 flex gap-3">
          <div className="flex-1 bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-3 text-center">
            <p className="text-2xl font-bold text-gray-900 dark:text-white">{accounts.length}</p>
            <p className="text-xs text-gray-400 dark:text-gray-500">аккаунтов</p>
          </div>
          <div className="flex-1 bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-3 text-center">
            <p className="text-2xl font-bold text-green-600 dark:text-green-400">{validAccounts.length}</p>
            <p className="text-xs text-gray-400 dark:text-gray-500">активных</p>
          </div>
          <div className="flex-1 bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-3 text-center">
            <p className="text-2xl font-bold text-blue-600 dark:text-blue-400">{aggregated.length}</p>
            <p className="text-xs text-gray-400 dark:text-gray-500">моделей</p>
          </div>
        </div>

        {/* Model Quotas */}
        <div className="mx-4">
          <div className="flex items-center justify-between mb-3">
            <h2 className="font-semibold text-gray-900 dark:text-white">Лимиты по моделям</h2>
            <button
              onClick={refreshQuotas}
              disabled={quotaLoading}
              className="text-xs text-blue-600 dark:text-blue-400 active:opacity-60 disabled:opacity-40"
            >
              {quotaLoading ? 'Загрузка...' : 'Обновить'}
            </button>
          </div>

          {aggregated.length === 0 && !quotaLoading && (
            <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-6 text-center">
              <p className="text-gray-400 dark:text-gray-500 text-sm">
                {accounts.length === 0
                  ? 'Добавьте аккаунты для просмотра лимитов'
                  : 'Нажмите «Обновить» для загрузки квот'}
              </p>
            </div>
          )}

          {quotaLoading && aggregated.length === 0 && (
            <div className="space-y-3">
              {[1, 2, 3].map((i) => (
                <div key={i} className="bg-white dark:bg-gray-800 rounded-2xl h-20 animate-pulse border border-gray-100 dark:border-gray-700" />
              ))}
            </div>
          )}

          <div className="space-y-3">
            {aggregated.map((q) => (
              <QuotaCard key={q.model} quota={q} />
            ))}
          </div>
        </div>

        {/* Quick actions */}
        {accounts.length > 0 && (
          <div className="mx-4">
            <button
              onClick={handleClearAll}
              disabled={clearingLimits}
              className="w-full py-3 rounded-2xl border border-orange-200 dark:border-orange-800 text-orange-600 dark:text-orange-400 text-sm font-medium active:bg-orange-50 dark:active:bg-orange-900/20 disabled:opacity-50"
            >
              {clearingLimits ? 'Очищаем...' : '⚡ Сбросить все rate limits'}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
