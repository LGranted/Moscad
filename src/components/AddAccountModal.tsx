
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
