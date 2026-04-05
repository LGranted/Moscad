
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
