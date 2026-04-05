
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
