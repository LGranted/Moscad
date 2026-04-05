import React, { useEffect, useState, useCallback } from 'react';
import { useAccountStore } from '../stores/useAccountStore';
import { useConfigStore } from '../stores/useConfigStore';
import { AddAccountModal } from '../components/AddAccountModal';
import { clearAllProxyRateLimits } from '../utils/request';
import type { Account, AccountQuota } from '../types';

// ─── AccountCard ─────────────────────────────────────────────────────────────

interface AccountCardProps {
  account: Account;
  quota?: AccountQuota;
  onDelete: () => void;
  onSetCurrent: () => void;
  onClearLimit: () => void;
}

const AccountCard: React.FC<AccountCardProps> = ({
  account,
  quota,
  onDelete,
  onSetCurrent,
  onClearLimit,
}) => {
  const [expanded, setExpanded] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-2xl border transition-colors ${
      account.is_current
        ? 'border-blue-300 dark:border-blue-600'
        : 'border-gray-100 dark:border-gray-700'
    }`}>
      {/* Main row */}
      <button
        className="w-full flex items-center gap-3 p-4 text-left active:bg-gray-50 dark:active:bg-gray-700/50 rounded-2xl"
        onClick={() => setExpanded((e) => !e)}
      >
        {/* Avatar */}
        <div className={`w-10 h-10 rounded-full flex items-center justify-center text-white text-sm font-bold shrink-0 ${
          !account.disabled ? 'bg-blue-500' : 'bg-gray-400 dark:bg-gray-600'
        }`}>
          {account.email?.[0]?.toUpperCase() ?? '?'}
        </div>

        <div className="flex-1 min-w-0">
          <p className="text-sm font-semibold text-gray-900 dark:text-white truncate">{account.email}</p>
          <div className="flex items-center gap-2 mt-0.5">
            {account.is_current && (
              <span className="text-xs text-blue-600 dark:text-blue-400 font-medium">● Текущий</span>
            )}
            <span className={`text-xs ${!account.disabled ? 'text-green-500' : 'text-red-500'}`}>
              {!account.disabled ? 'Активен' : 'Отключён'}
            </span>
          </div>
        </div>

        <svg
          viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}
          className={`w-4 h-4 text-gray-400 shrink-0 transition-transform ${expanded ? 'rotate-180' : ''}`}
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Expanded section */}
      {expanded && (
        <div className="px-4 pb-4 space-y-3 border-t border-gray-100 dark:border-gray-700 pt-3">
          {/* Quota summary */}
          {quota?.error ? (
            <p className="text-xs text-red-500 dark:text-red-400">{quota.error}</p>
          ) : quota?.quotas && quota.quotas.length > 0 ? (
            <div className="space-y-1">
              {quota.quotas.slice(0, 5).map((mq) => (
                <div key={mq.model} className="flex items-center justify-between">
                  <span className="text-xs text-gray-500 dark:text-gray-400 truncate max-w-[60%]">{mq.model}</span>
                  <span className="text-xs font-medium text-gray-700 dark:text-gray-300">
                    {mq.requests_remaining != null ? `${mq.requests_remaining} req` : '—'}
                  </span>
                </div>
              ))}
              {quota.quotas.length > 5 && (
                <p className="text-xs text-gray-400">+{quota.quotas.length - 5} ещё...</p>
              )}
            </div>
          ) : (
            <p className="text-xs text-gray-400 dark:text-gray-500">Квоты не загружены</p>
          )}

          {/* Action buttons */}
          <div className="flex gap-2 flex-wrap">
            {!account.is_current && (
              <button
                onClick={onSetCurrent}
                className="flex-1 py-2 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 text-xs font-medium active:opacity-70"
              >
                Сделать текущим
              </button>
            )}
            <button
              onClick={onClearLimit}
              className="flex-1 py-2 rounded-xl bg-orange-50 dark:bg-orange-900/20 text-orange-600 dark:text-orange-400 text-xs font-medium active:opacity-70"
            >
              Сбросить лимит
            </button>
            {confirmDelete ? (
              <div className="w-full flex gap-2">
                <button
                  onClick={() => setConfirmDelete(false)}
                  className="flex-1 py-2 rounded-xl bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 text-xs font-medium"
                >
                  Отмена
                </button>
                <button
                  onClick={onDelete}
                  className="flex-1 py-2 rounded-xl bg-red-500 text-white text-xs font-medium active:bg-red-600"
                >
                  Удалить
                </button>
              </div>
            ) : (
              <button
                onClick={() => setConfirmDelete(true)}
                className="flex-1 py-2 rounded-xl bg-red-50 dark:bg-red-900/20 text-red-500 text-xs font-medium active:opacity-70"
              >
                Удалить
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

// ─── Accounts Page ───────────────────────────────────────────────────────────

export const Accounts: React.FC = () => {
  const {
    accounts, quotas, loading,
    fetchAccounts, deleteAccount, switchAccount, clearRateLimit, fetchQuotaForAll,
  } = useAccountStore();
  const { config, saveConfig } = useConfigStore();

  const [showAdd, setShowAdd] = useState(false);
  const [quotaLoading, setQuotaLoading] = useState(false);

  const refresh = useCallback(async () => {
    await fetchAccounts();
  }, [fetchAccounts]);

  const refreshQuotas = useCallback(async () => {
    setQuotaLoading(true);
    try { await fetchQuotaForAll(); }
    finally { setQuotaLoading(false); }
  }, [fetchQuotaForAll]);

  useEffect(() => {
    refresh();
  }, []);

  useEffect(() => {
    if (accounts.length > 0) refreshQuotas();
  }, [accounts.length]);

  const handleModalClose = useCallback(() => {
    setShowAdd(false);
    refresh();
  }, [refresh]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 pb-24">
      {/* Header */}
      <header className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-950 px-4 pt-safe pt-4 pb-3 flex items-center justify-between border-b border-gray-100 dark:border-gray-800">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">Аккаунты</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={refresh}
            className="p-2 rounded-xl active:bg-gray-200 dark:active:bg-gray-700"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className={`w-5 h-5 text-gray-500 dark:text-gray-400 ${loading ? 'animate-spin' : ''}`}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          <button
            onClick={() => setShowAdd(true)}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-blue-600 text-white text-sm font-semibold active:bg-blue-700"
          >
            <span>+</span>
            <span>Добавить</span>
          </button>
        </div>
      </header>

      <div className="px-4 pt-4 space-y-4">
        {/* Rotation settings */}
        <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 p-4">
          <div className="flex items-center justify-between mb-3">
            <div>
              <p className="text-sm font-semibold text-gray-900 dark:text-white">Ротация аккаунтов</p>
              <p className="text-xs text-gray-400 dark:text-gray-500">Автоматически переключать при лимитах</p>
            </div>
            <button
              onClick={() => saveConfig({ rotation_enabled: !config.rotation_enabled })}
              className={`relative w-12 h-6 rounded-full transition-colors ${
                config.rotation_enabled ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'
              }`}
            >
              <span className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform ${
                config.rotation_enabled ? 'translate-x-6' : ''
              }`} />
            </button>
          </div>

          {config.rotation_enabled && (
            <div className="flex gap-2">
              {(['round-robin', 'by-limit'] as const).map((s) => (
                <button
                  key={s}
                  onClick={() => saveConfig({ rotation_strategy: s })}
                  className={`flex-1 py-2 rounded-xl text-xs font-medium transition-colors ${
                    config.rotation_strategy === s
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300'
                  }`}
                >
                  {s === 'round-robin' ? 'По очереди' : 'По лимитам'}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Account list */}
        {accounts.length === 0 && !loading && (
          <div className="text-center py-12">
            <div className="text-5xl mb-3">👤</div>
            <p className="text-gray-500 dark:text-gray-400 font-medium">Нет аккаунтов</p>
            <p className="text-sm text-gray-400 dark:text-gray-500 mt-1">Нажмите «Добавить», чтобы начать</p>
          </div>
        )}

        {loading && accounts.length === 0 && (
          <div className="space-y-3">
            {[1, 2].map((i) => (
              <div key={i} className="bg-white dark:bg-gray-800 rounded-2xl h-20 animate-pulse border border-gray-100 dark:border-gray-700" />
            ))}
          </div>
        )}

        <div className="space-y-3">
          {accounts.map((acc) => (
            <AccountCard
              key={acc.id}
              account={acc}
              quota={quotas[acc.id]}
              onDelete={() => deleteAccount(acc.id)}
              onSetCurrent={() => switchAccount(acc.id)}
              onClearLimit={() => clearRateLimit(acc.id)}
            />
          ))}
        </div>

        {accounts.length > 0 && (
          <button
            onClick={refreshQuotas}
            disabled={quotaLoading}
            className="w-full py-3 rounded-2xl border border-gray-200 dark:border-gray-700 text-gray-500 dark:text-gray-400 text-sm active:bg-gray-100 dark:active:bg-gray-800 disabled:opacity-40"
          >
            {quotaLoading ? 'Обновление квот...' : 'Обновить квоты'}
          </button>
        )}
      </div>

      {showAdd && <AddAccountModal onClose={handleModalClose} />}
    </div>
  );
};
