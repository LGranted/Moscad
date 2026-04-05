import React, { useState, useCallback } from 'react';
import { open as openUrl } from '@tauri-apps/plugin-shell';
import { startOAuthFlow, submitOAuthCode, cancelOAuthFlow } from '../utils/request';
import { useAccountStore } from '../stores/useAccountStore';

interface Props {
  onClose: () => void;
}

type Step = 'idle' | 'oauth_started' | 'submitting' | 'done' | 'error';

export const AddAccountModal: React.FC<Props> = ({ onClose }) => {
  const [step, setStep] = useState<Step>('idle');
  const [oauthUrl, setOauthUrl] = useState('');
  const [oauthState, setOauthState] = useState('');
  const [code, setCode] = useState('');
  const [errorMsg, setErrorMsg] = useState('');

  const fetchAccounts = useAccountStore((s) => s.fetchAccounts);

  const handleStartOAuth = useCallback(async () => {
    setErrorMsg('');
    try {
      const result = await startOAuthFlow();
      setOauthUrl(result.url);
      setOauthState(result.state);
      setStep('oauth_started');
      try { await openUrl(result.url); } catch { /* user opens manually */ }
    } catch (e) {
      setErrorMsg(String(e));
    }
  }, []);

  const handleSubmitCode = useCallback(async () => {
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
    try { await cancelOAuthFlow(); } catch { /* ignore */ }
    onClose();
  }, [onClose]);

  const copyUrl = useCallback(() => {
    navigator.clipboard.writeText(oauthUrl).catch(() => {});
  }, [oauthUrl]);

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center bg-black/50 dark:bg-black/70">
      <div className="w-full max-w-lg bg-white dark:bg-gray-900 rounded-t-2xl px-5 pt-5 pb-safe shadow-2xl">
        <div className="w-12 h-1 rounded-full bg-gray-300 dark:bg-gray-600 mx-auto mb-5" />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">Добавить аккаунт</h2>
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-5">Авторизация через Google OAuth 2.0</p>

        {step === 'done' && (
          <div className="flex items-center gap-3 py-6 justify-center">
            <span className="text-green-500 text-3xl">✓</span>
            <span className="text-green-600 dark:text-green-400 font-medium">Аккаунт добавлен!</span>
          </div>
        )}

        {step !== 'done' && (
          <>
            <div className={`mb-5 rounded-xl p-4 ${step === 'idle' ? 'bg-blue-50 dark:bg-blue-900/20' : 'bg-gray-50 dark:bg-gray-800'}`}>
              <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                1. Нажмите «Начать OAuth» — браузер откроется автоматически. Если нет, скопируйте ссылку вручную.
              </p>
              {step === 'idle' ? (
                <button onClick={handleStartOAuth} className="w-full py-3 rounded-xl bg-blue-600 text-white font-semibold text-sm active:bg-blue-700">
                  Начать OAuth
                </button>
              ) : (
                <div className="flex gap-2">
                  <div className="flex-1 text-xs bg-white dark:bg-gray-700 rounded-lg px-3 py-2 text-gray-600 dark:text-gray-300 truncate border border-gray-200 dark:border-gray-600">
                    {oauthUrl}
                  </div>
                  <button onClick={copyUrl} className="px-3 py-2 rounded-lg bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 text-xs font-medium active:opacity-70">
                    Копировать
                  </button>
                </div>
              )}
            </div>

            {step !== 'idle' && (
              <div className="mb-5 rounded-xl p-4 bg-gray-50 dark:bg-gray-800">
                <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                  2. Авторизуйтесь и скопируйте код из URL (<code className="text-blue-600 dark:text-blue-400">code=...</code>).
                </p>
                <input
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  placeholder="Вставьте код сюда..."
                  className="w-full rounded-xl border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-4 py-3 text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 mb-3"
                />
                <button
                  onClick={handleSubmitCode}
                  disabled={!code.trim() || step === 'submitting'}
                  className="w-full py-3 rounded-xl bg-green-600 disabled:bg-gray-300 dark:disabled:bg-gray-700 text-white font-semibold text-sm active:bg-green-700 disabled:text-gray-400"
                >
                  {step === 'submitting' ? 'Проверяем...' : 'Подтвердить'}
                </button>
              </div>
            )}

            {errorMsg && (
              <div className="mb-4 rounded-xl bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 px-4 py-3">
                <p className="text-sm text-red-600 dark:text-red-400">{errorMsg}</p>
              </div>
            )}

            <button onClick={handleCancel} className="w-full py-3 rounded-xl text-gray-500 dark:text-gray-400 text-sm font-medium active:text-gray-700 mb-1">
              Отмена
            </button>
          </>
        )}
      </div>
    </div>
  );
};
