#!/usr/bin/env python3
"""
fix2.py — Финальный точечный фикс оставшихся 3 ошибок
Запускать из корня: python3 fix2.py
"""
import re, os

def read(p):
    if not os.path.exists(p):
        print(f"  НЕ НАЙДЕН: {p}"); return None
    with open(p, encoding="utf-8") as f: return f.read()

def write(p, c, orig):
    if c == orig: print(f"  [без изменений] {p}")
    else:
        with open(p, "w", encoding="utf-8") as f: f.write(c)
        print(f"  [ИСПРАВЛЕН] {p}")

NA = '#[cfg(not(target_os = "android"))]'
B  = "src-tauri/src"

# ── 1. modules/mod.rs ────────────────────────────────────────────────────────
# config, logger, log_bridge нужны на Android — убираем cfg-guard с них
print("[1] modules/mod.rs — освобождаем config / logger / log_bridge")
p = B + "/modules/mod.rs"
c = read(p)
if c:
    orig = c
    for mod_name in ["config", "logger", "log_bridge"]:
        # Убираем #[cfg(not(target_os = "android"))] стоящий строго ПЕРЕД pub mod <name>;
        c = re.sub(
            rf'#\[cfg\(not\(target_os\s*=\s*"android"\)\)\]\s*\n(\s*pub mod {mod_name};)',
            r'\1',
            c,
        )
        # На случай если guard на той же строке через пробел
        c = re.sub(
            rf'#\[cfg\(not\(target_os\s*=\s*"android"\)\)\]\s+(pub mod {mod_name};)',
            r'\1',
            c,
        )
    write(p, c, orig)
    print("     Итог mod.rs:")
    for line in c.splitlines():
        if "pub mod" in line or "cfg" in line:
            print("    ", line)

# ── 2. utils/http.rs — unresolved import crate::modules::config ──────────────
# Если в http.rs стоит use crate::modules::config::load_app_config;
# и config оказался под cfg — просто убедимся что импорт без cfg-guard
print("\n[2] utils/http.rs — импорт modules::config без cfg-guard")
p = B + "/utils/http.rs"
c = read(p)
if c:
    orig = c
    # Убираем любой cfg-guard перед use crate::modules::config
    c = re.sub(
        r'#\[cfg\([^\]]*\)\]\s*\n(\s*use crate::modules::config)',
        r'\1',
        c,
    )
    write(p, c, orig)

# ── 3. Ищем где используется modules::logger и log_bridge — снимаем cfg ──────
print("\n[3] Файлы с modules::logger и modules::log_bridge — снимаем cfg-guard")
import glob
for filepath in glob.glob(B + "/**/*.rs", recursive=True):
    c = read(filepath)
    if not c: continue
    if "modules::log_bridge" not in c and "modules::logger" not in c: continue
    orig = c
    for mod_name in ["logger", "log_bridge"]:
        # Снимаем guard с use-импорта этих модулей
        c = re.sub(
            rf'#\[cfg\([^\]]*\)\]\s*\n(\s*(?:use|pub use) crate::modules::{mod_name})',
            r'\1',
            c,
        )
        c = re.sub(
            rf'#\[cfg\([^\]]*\)\]\s+(\s*(?:use|pub use) crate::modules::{mod_name})',
            r'\1',
            c,
        )
    write(filepath, c, orig)

print("""
Готово! Выполни:

  cd ~/Antigravity-Manager
  git add src-tauri/src/
  git commit -m "fix: ungate config/logger/log_bridge on Android"
  git push
""")
