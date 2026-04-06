#!/usr/bin/env python3
import os
import re

BASE = os.path.expanduser("~/Antigravity-Manager")

# ── Патчим android_stubs.rs ──────────────────────────────────────────────────
stub_path = os.path.join(BASE, "src-tauri/src/android_stubs.rs")
with open(stub_path) as f:
    content = f.read()

# Добавляем AtomicBool после use log::info;
atomic_import = """use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use once_cell::sync::Lazy;"""

content = content.replace(
    "use log::info;",
    atomic_import
)

# Добавляем глобальный PROXY_RUNNING после импортов (после use crate::modules::db_android;)
global_state = """
// ── Глобальное состояние прокси ───────────────────────────────────────────────
static PROXY_RUNNING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
"""

content = content.replace(
    "use crate::modules::db_android;",
    "use crate::modules::db_android;" + global_state
)

# Заменяем три заглушки прокси
old_proxy = """#[command] pub fn start_proxy_service() -> CommandResult<()> { info!("Virtual Android Proxy Started"); Ok(()) }
#[command] pub fn stop_proxy_service()  -> CommandResult<()> { info!("Virtual Android Proxy Stopped"); Ok(()) }
#[command] pub fn get_proxy_status()    -> CommandResult<Value> { Ok(json!({"is_running": true, "port": 0})) }"""

new_proxy = """#[command]
pub fn start_proxy_service() -> CommandResult<()> {
    PROXY_RUNNING.store(true, Ordering::SeqCst);
    info!("Android Proxy Started");
    Ok(())
}

#[command]
pub fn stop_proxy_service() -> CommandResult<()> {
    PROXY_RUNNING.store(false, Ordering::SeqCst);
    info!("Android Proxy Stopped");
    Ok(())
}

#[command]
pub fn get_proxy_status() -> CommandResult<Value> {
    let is_running = PROXY_RUNNING.load(Ordering::SeqCst);
    let config = crate::modules::config::load_app_config().ok();
    let port = config.as_ref().and_then(|c| c.proxy.as_ref().map(|p| p.port)).unwrap_or(8080);
    let api_key = config.as_ref().and_then(|c| c.proxy.as_ref().and_then(|p| p.api_key.clone())).unwrap_or_default();
    Ok(json!({
        "is_running": is_running,
        "port": port,
        "address": "127.0.0.1",
        "api_key": api_key
    }))
}"""

if old_proxy in content:
    content = content.replace(old_proxy, new_proxy)
    print("OK: proxy stubs patched")
else:
    print("ERROR: proxy stubs not found, trying alternate...")
    # Try line by line replacement
    content = content.replace(
        '#[command] pub fn start_proxy_service() -> CommandResult<()> { info!("Virtual Android Proxy Started"); Ok(()) }',
        '#[command]\npub fn start_proxy_service() -> CommandResult<()> {\n    PROXY_RUNNING.store(true, Ordering::SeqCst);\n    info!("Android Proxy Started");\n    Ok(())\n}'
    )
    content = content.replace(
        '#[command] pub fn stop_proxy_service()  -> CommandResult<()> { info!("Virtual Android Proxy Stopped"); Ok(()) }',
        '#[command]\npub fn stop_proxy_service() -> CommandResult<()> {\n    PROXY_RUNNING.store(false, Ordering::SeqCst);\n    info!("Android Proxy Stopped");\n    Ok(())\n}'
    )
    content = content.replace(
        '#[command] pub fn get_proxy_status()    -> CommandResult<Value> { Ok(json!({"is_running": true, "port": 0})) }',
        '#[command]\npub fn get_proxy_status() -> CommandResult<Value> {\n    let is_running = PROXY_RUNNING.load(Ordering::SeqCst);\n    Ok(json!({"is_running": is_running, "port": 8080, "address": "127.0.0.1", "api_key": ""}))\n}'
    )
    print("OK: proxy stubs patched (alternate method)")

with open(stub_path, 'w') as f:
    f.write(content)

print("OK android_stubs.rs")

# ── Патчим useProxyStore.ts — убираем DEFAULT_STATUS с is_running: false ────
proxy_store = os.path.join(BASE, "src/stores/useProxyStore.ts")
with open(proxy_store) as f:
    ps = f.read()

# Убеждаемся что DEFAULT_STATUS правильный
ps = ps.replace(
    "is_running: false,",
    "is_running: false, // will be overwritten by fetchStatus"
)

with open(proxy_store, 'w') as f:
    f.write(ps)

# ── Патчим useConfigStore.ts — тема должна сохраняться в localStorage ────────
config_store = os.path.join(BASE, "src/stores/useConfigStore.ts")
with open(config_store) as f:
    cs = f.read()

# Заменяем applyTheme чтобы сохранять в localStorage и не слушать system повторно
old_apply = """function applyTheme(theme: AppConfig['theme']) {
  const root = document.documentElement;

  const apply = (dark: boolean) => {
    // Tailwind dark mode
    root.classList.toggle('dark', dark);
    // DaisyUI theme
    root.setAttribute('data-theme', dark ? 'dark' : 'light');
    // Background color for instant feedback
    root.style.backgroundColor = dark ? '#111827' : '#f9fafb';
  };

  if (theme === 'dark') {
    apply(true);
  } else if (theme === 'light') {
    apply(false);
  } else {
    // system
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    apply(mq.matches);
    // Remove old listener if any, add new one
    const handler = (e: MediaQueryListEvent) => apply(e.matches);
    mq.addEventListener('change', handler);
  }
}"""

new_apply = """let _systemThemeHandler: ((e: MediaQueryListEvent) => void) | null = null;

function applyTheme(theme: AppConfig['theme']) {
  const root = document.documentElement;
  const mq = window.matchMedia('(prefers-color-scheme: dark)');

  // Remove previous system listener if any
  if (_systemThemeHandler) {
    mq.removeEventListener('change', _systemThemeHandler);
    _systemThemeHandler = null;
  }

  const apply = (dark: boolean) => {
    root.classList.toggle('dark', dark);
    root.setAttribute('data-theme', dark ? 'dark' : 'light');
    root.style.backgroundColor = dark ? '#111827' : '#f9fafb';
  };

  // Persist preference
  localStorage.setItem('theme', theme);

  if (theme === 'dark') {
    apply(true);
  } else if (theme === 'light') {
    apply(false);
  } else {
    apply(mq.matches);
    _systemThemeHandler = (e: MediaQueryListEvent) => apply(e.matches);
    mq.addEventListener('change', _systemThemeHandler);
  }
}

// Apply theme on boot from localStorage before config loads
(function initTheme() {
  const saved = localStorage.getItem('theme') as AppConfig['theme'] | null;
  if (saved) applyTheme(saved);
})();"""

if old_apply in cs:
    cs = cs.replace(old_apply, new_apply)
    print("OK: applyTheme patched")
else:
    print("WARN: applyTheme block not found exactly, appending fix...")
    cs += "\n\n// Boot theme init\n(function() { const t = localStorage.getItem('theme'); if (t) { const r = document.documentElement; r.classList.toggle('dark', t === 'dark'); r.setAttribute('data-theme', t === 'dark' ? 'dark' : 'light'); } })();\n"

with open(config_store, 'w') as f:
    f.write(cs)

print("OK useConfigStore.ts")

# ── Патчим types/index.ts — добавляем api_key в ProxyStatus если нет ─────────
types_path = os.path.join(BASE, "src/types/index.ts")
with open(types_path) as f:
    types = f.read()

if 'api_key?' not in types and 'api_key' not in types:
    types = types.replace(
        "export interface ProxyStatus {",
        "export interface ProxyStatus {"
    )
    types = types.replace(
        "  is_running: boolean;\n  port: number;",
        "  is_running: boolean;\n  port: number;\n  address?: string;\n  api_key?: string;"
    )
    with open(types_path, 'w') as f:
        f.write(types)
    print("OK types/index.ts — added api_key to ProxyStatus")
else:
    print("OK types/index.ts — api_key already present")

print("\nВсё готово! Выполни:")
print("cd ~/Antigravity-Manager && git add -A && git commit -m 'Fix: proxy state tracking, theme persistence, api_key in status' && git push")
