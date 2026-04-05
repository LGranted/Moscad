
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
