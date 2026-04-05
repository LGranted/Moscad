import React, { useEffect, useState, useCallback, useRef } from 'react';
import { getLogsFiltered, getLogsCount } from '../utils/request';
import type { LogEntry } from '../types';

const METHOD_COLORS: Record<string, string> = {
  GET: 'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-400',
  POST: 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-400',
  PUT: 'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-400',
  DELETE: 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-400',
  PATCH: 'bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-400',
};

function statusColor(code: number): string {
  if (code >= 500) return 'text-red-600 dark:text-red-400';
  if (code >= 400) return 'text-orange-600 dark:text-orange-400';
  if (code >= 300) return 'text-yellow-600 dark:text-yellow-400';
  if (code >= 200) return 'text-green-600 dark:text-green-400';
  return 'text-gray-500';
}

function formatTime(ts: string): string {
  try {
    const d = new Date(ts);
    return d.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  } catch {
    return ts;
  }
}

function truncateUrl(url: string, len = 48): string {
  return url.length > len ? `${url.slice(0, len)}…` : url;
}

const LogRow: React.FC<{ entry: LogEntry }> = ({ entry }) => {
  const [expanded, setExpanded] = useState(false);
  const isError = entry.status_code >= 400 || !!entry.error;

  return (
    <div
      className={`rounded-2xl border transition-colors ${
        isError
          ? 'bg-red-50 dark:bg-red-900/10 border-red-200 dark:border-red-800'
          : 'bg-white dark:bg-gray-800 border-gray-100 dark:border-gray-700'
      }`}
    >
      <button
        className="w-full p-3 text-left active:opacity-80"
        onClick={() => setExpanded((e) => !e)}
      >
        <div className="flex items-center gap-2 mb-1">
          <span className={`text-[10px] font-bold px-1.5 py-0.5 rounded ${METHOD_COLORS[entry.method] ?? 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400'}`}>
            {entry.method}
          </span>
          <span className={`text-xs font-bold ${statusColor(entry.status_code)}`}>
            {entry.status_code}
          </span>
          {entry.duration_ms != null && (
            <span className="text-[10px] text-gray-400 dark:text-gray-500">{entry.duration_ms}ms</span>
          )}
          <span className="ml-auto text-[10px] text-gray-400 dark:text-gray-500">
            {formatTime(entry.timestamp)}
          </span>
        </div>
        <p className="text-xs text-gray-700 dark:text-gray-300 font-mono leading-tight">
          {truncateUrl(entry.url)}
        </p>
        {entry.account_email && (
          <p className="text-[10px] text-gray-400 dark:text-gray-500 mt-0.5">{entry.account_email}</p>
        )}
        {entry.error && (
          <p className="text-xs text-red-600 dark:text-red-400 mt-1 truncate">{entry.error}</p>
        )}
      </button>

      {expanded && (
        <div className="px-3 pb-3 border-t border-gray-100 dark:border-gray-700 pt-2 space-y-2">
          {entry.model && (
            <p className="text-xs text-gray-600 dark:text-gray-400">
              <span className="font-medium">Модель:</span> {entry.model}
            </p>
          )}
          <p className="text-xs text-gray-600 dark:text-gray-400 font-mono break-all">{entry.url}</p>
          {entry.error && (
            <div className="rounded-xl bg-red-100 dark:bg-red-900/30 p-2">
              <p className="text-xs text-red-700 dark:text-red-300 font-mono break-all">{entry.error}</p>
            </div>
          )}
          {entry.request_body && (
            <details className="text-xs">
              <summary className="text-gray-500 dark:text-gray-400 cursor-pointer">Тело запроса</summary>
              <pre className="mt-1 bg-gray-100 dark:bg-gray-700 rounded-xl p-2 overflow-x-auto text-gray-700 dark:text-gray-300 text-[10px]">
                {entry.request_body.slice(0, 500)}
              </pre>
            </details>
          )}
          {entry.response_body && (
            <details className="text-xs">
              <summary className="text-gray-500 dark:text-gray-400 cursor-pointer">Тело ответа</summary>
              <pre className="mt-1 bg-gray-100 dark:bg-gray-700 rounded-xl p-2 overflow-x-auto text-gray-700 dark:text-gray-300 text-[10px]">
                {entry.response_body.slice(0, 500)}
              </pre>
            </details>
          )}
        </div>
      )}
    </div>
  );
};

// ─── Logs Page ────────────────────────────────────────────────────────────────

export const Logs: React.FC = () => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [errorsOnly, setErrorsOnly] = useState(false);
  const [search, setSearch] = useState('');
  const [totalCount, setTotalCount] = useState(0);
  const [limit] = useState(200);

  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchLogs = useCallback(async () => {
    setLoading(true);
    try {
      const [entries, count] = await Promise.all([
        getLogsFiltered(errorsOnly, search, limit),
        getLogsCount(errorsOnly, search),
      ]);
      setLogs(entries);
      setTotalCount(count);
    } catch {
      // fail silently on auto-refresh
    } finally {
      setLoading(false);
    }
  }, [errorsOnly, search, limit]);

  useEffect(() => {
    fetchLogs();
  }, [errorsOnly, search]);

  // Auto-refresh every 5s
  useEffect(() => {
    if (intervalRef.current) clearInterval(intervalRef.current);
    intervalRef.current = setInterval(fetchLogs, 5_000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [fetchLogs]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 pb-24">
      {/* Header */}
      <header className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-950 px-4 pt-safe pt-4 pb-3 border-b border-gray-100 dark:border-gray-800">
        <div className="flex items-center justify-between mb-3">
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">
            Логи
            {totalCount > 0 && (
              <span className="ml-2 text-sm font-normal text-gray-400 dark:text-gray-500">({totalCount})</span>
            )}
          </h1>
          <div className="flex items-center gap-2">
            {loading && (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-4 h-4 text-blue-500 animate-spin">
                <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            )}
            <button
              onClick={fetchLogs}
              className="p-2 rounded-xl active:bg-gray-200 dark:active:bg-gray-700"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-5 h-5 text-gray-500 dark:text-gray-400">
                <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </button>
          </div>
        </div>

        {/* Search */}
        <div className="flex gap-2">
          <div className="flex-1 flex items-center gap-2 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 px-3 py-2">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="w-4 h-4 text-gray-400 shrink-0">
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Поиск в логах..."
              className="flex-1 bg-transparent text-sm text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none"
            />
            {search && (
              <button onClick={() => setSearch('')} className="text-gray-400 active:text-gray-600">✕</button>
            )}
          </div>

          <button
            onClick={() => setErrorsOnly((e) => !e)}
            className={`px-3 py-2 rounded-xl text-xs font-semibold transition-colors ${
              errorsOnly
                ? 'bg-red-500 text-white'
                : 'bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400'
            }`}
          >
            Ошибки
          </button>
        </div>
      </header>

      <div className="px-4 pt-4 space-y-2">
        {logs.length === 0 && !loading && (
          <div className="text-center py-12">
            <div className="text-5xl mb-3">📋</div>
            <p className="text-gray-500 dark:text-gray-400 font-medium">
              {errorsOnly || search ? 'Нет совпадений' : 'Логи пусты'}
            </p>
            <p className="text-sm text-gray-400 dark:text-gray-500 mt-1">
              Логи появятся после запуска прокси
            </p>
          </div>
        )}

        {logs.map((entry) => (
          <LogRow key={entry.id} entry={entry} />
        ))}
      </div>
    </div>
  );
};
