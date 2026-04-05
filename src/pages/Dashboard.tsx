
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
