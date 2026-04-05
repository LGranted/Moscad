import React, { useEffect, useState } from 'react';
import { useConfigStore } from '../stores/useConfigStore';
import { clearAllProxyRateLimits } from '../utils/request';
import type { AppConfig } from '../types';

// ─── Helper components ────────────────────────────────────────────────────────

interface ToggleRowProps {
  label: string;
  description?: string;
  value: boolean;
  onChange: (v: boolean) => void;
}

const ToggleRow: React.FC<ToggleRowProps> = ({ label, description, value, onChange }) => (
  <div className="flex items-center justify-between py-3.5 border-b border-gray-100 dark:border-gray-700 last:border-0">
    <div className="flex-1 mr-3">
      <p className="text-sm font-medium text-gray-900 dark:text-white">{label}</p>
      {description && <p className="text-xs text-gray-400 dark:text-gray-500 mt-0.5">{description}</p>}
    </div>
    <button
      onClick={() => onChange(!value)}
      className={`relative w-12 h-6 rounded-full transition-colors shrink-0 ${value ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'}`}
    >
      <span className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform ${value ? 'translate-x-6' : ''}`} />
    </button>
  </div>
);

interface SelectRowProps {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (v: string) => void;
}

const SelectRow: React.FC<SelectRowProps> = ({ label, value, options, onChange }) => (
  <div className="flex items-center justify-between py-3.5 border-b border-gray-100 dark:border-gray-700 last:border-0">
    <p className="text-sm font-medium text-gray-900 dark:text-white">{label}</p>
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white text-sm rounded-lg px-2 py-1.5 border-none focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      {options.map((o) => (
        <option key={o.value} value={o.value}>{o.label}</option>
      ))}
    </select>
  </div>
);

interface NumberRowProps {
  label: string;
  description?: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  onChange: (v: number) => void;
}

const NumberRow: React.FC<NumberRowProps> = ({ label, description, value, min, max, step = 1, onChange }) => {
  const [local, setLocal] = useState(String(value));

  useEffect(() => setLocal(String(value)), [value]);

  return (
    <div className="flex items-center justify-between py-3.5 border-b border-gray-100 dark:border-gray-700 last:border-0">
      <div className="flex-1 mr-3">
        <p className="text-sm font-medium text-gray-900 dark:text-white">{label}</p>
        {description && <p className="text-xs text-gray-400 dark:text-gray-500 mt-0.5">{description}</p>}
      </div>
      <input
        type="number"
        value={local}
        min={min}
        max={max}
        step={step}
        onChange={(e) => setLocal(e.target.value)}
        onBlur={() => {
          const n = Number(local);
          if (!isNaN(n)) onChange(n);
        }}
        className="w-24 bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white text-sm rounded-lg px-3 py-1.5 focus:outline-none focus:ring-2 focus:ring-blue-500 text-right"
      />
    </div>
  );
};

interface SectionProps {
  title: string;
  children: React.ReactNode;
}

const Section: React.FC<SectionProps> = ({ title, children }) => (
  <div className="mx-4 mb-4">
    <h2 className="text-xs font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider px-1 mb-2">{title}</h2>
    <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 px-4">
      {children}
    </div>
  </div>
);

// ─── Settings Page ────────────────────────────────────────────────────────────

export const Settings: React.FC = () => {
  const { config, saveConfig, fetchConfig } = useConfigStore();
  const [saving, setSaving] = useState(false);
  const [clearingLimits, setClearingLimits] = useState(false);
  const [savedMsg, setSavedMsg] = useState(false);

  useEffect(() => {
    fetchConfig();
  }, []);

  const update = async (partial: Partial<AppConfig>) => {
    setSaving(true);
    await saveConfig(partial);
    setSaving(false);
    setSavedMsg(true);
    setTimeout(() => setSavedMsg(false), 1500);
  };

  const handleClearAll = async () => {
    setClearingLimits(true);
    try { await clearAllProxyRateLimits(); }
    finally { setClearingLimits(false); }
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 pb-24">
      {/* Header */}
      <header className="sticky top-0 z-10 bg-gray-50 dark:bg-gray-950 px-4 pt-safe pt-4 pb-3 flex items-center justify-between border-b border-gray-100 dark:border-gray-800">
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">Настройки</h1>
        {(saving || savedMsg) && (
          <span className="text-xs text-blue-600 dark:text-blue-400 font-medium">
            {saving ? 'Сохранение...' : '✓ Сохранено'}
          </span>
        )}
      </header>

      <div className="pt-4">
        {/* Appearance */}
        <Section title="Внешний вид">
          <SelectRow
            label="Тема"
            value={config.theme}
            options={[
              { value: 'system', label: 'Системная' },
              { value: 'light', label: 'Светлая' },
              { value: 'dark', label: 'Тёмная' },
            ]}
            onChange={(v) => update({ theme: v as AppConfig['theme'] })}
          />
          <SelectRow
            label="Язык"
            value={config.language}
            options={[
              { value: 'ru', label: 'Русский' },
              { value: 'en', label: 'English' },
              { value: 'zh', label: '中文' },
            ]}
            onChange={(v) => update({ language: v })}
          />
        </Section>

        {/* Proxy */}
        <Section title="Прокси">
          <ToggleRow
            label="Автозапуск при старте"
            description="Запускать прокси при открытии приложения"
            value={config.auto_start_proxy}
            onChange={(v) => update({ auto_start_proxy: v })}
          />
          <NumberRow
            label="Порт"
            description="Порт для прослушивания (8080 по умолчанию)"
            value={config.proxy_port}
            min={1024}
            max={65535}
            onChange={(v) => update({ proxy_port: v })}
          />
          <NumberRow
            label="Таймаут запроса (мс)"
            value={config.request_timeout_ms}
            min={1000}
            step={1000}
            onChange={(v) => update({ request_timeout_ms: v })}
          />
        </Section>

        {/* Retries */}
        <Section title="Повторные запросы">
          <ToggleRow
            label="Автоматические повторы"
            description="Повторять при ошибке 429 (rate limit)"
            value={config.retry_enabled}
            onChange={(v) => update({ retry_enabled: v })}
          />
          {config.retry_enabled && (
            <>
              <NumberRow
                label="Макс. повторов"
                value={config.retry_limit}
                min={0}
                max={20}
                onChange={(v) => update({ retry_limit: v })}
              />
              <NumberRow
                label="Задержка (мс)"
                description="Базовая задержка между попытками"
                value={config.retry_base_delay_ms}
                min={100}
                step={100}
                onChange={(v) => update({ retry_base_delay_ms: v })}
              />
            </>
          )}
          <div className="py-3.5">
            <button
              onClick={handleClearAll}
              disabled={clearingLimits}
              className="w-full py-2.5 rounded-xl bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 text-orange-600 dark:text-orange-400 text-sm font-medium active:opacity-70 disabled:opacity-40"
            >
              {clearingLimits ? 'Очищаем...' : '⚡ Очистить все rate limits'}
            </button>
          </div>
        </Section>

        {/* Logs */}
        <Section title="Логи">
          <ToggleRow
            label="Запись логов"
            value={config.log_enabled}
            onChange={(v) => update({ log_enabled: v })}
          />
          {config.log_enabled && (
            <SelectRow
              label="Уровень"
              value={config.log_level}
              options={[
                { value: 'debug', label: 'Debug' },
                { value: 'info', label: 'Info' },
                { value: 'warn', label: 'Warn' },
                { value: 'error', label: 'Error' },
              ]}
              onChange={(v) => update({ log_level: v as AppConfig['log_level'] })}
            />
          )}
        </Section>

        {/* Advanced */}
        <Section title="Дополнительно">
          <div className="py-3.5 border-b border-gray-100 dark:border-gray-700 last:border-0">
            <p className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">Upstream прокси (необязательно)</p>
            <input
              type="text"
              value={config.upstream_proxy ?? ''}
              onChange={(e) => update({ upstream_proxy: e.target.value || undefined })}
              placeholder="http://proxy:8888"
              className="w-full bg-gray-100 dark:bg-gray-700 rounded-xl px-3 py-2 text-sm text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </Section>

        {/* Version */}
        <div className="text-center py-4 pb-8">
          <p className="text-xs text-gray-400 dark:text-gray-600">Moscad · Google Antigravity Proxy</p>
          <p className="text-xs text-gray-300 dark:text-gray-700 mt-0.5">Android build</p>
        </div>
      </div>
    </div>
  );
};
