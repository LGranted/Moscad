import re
from pathlib import Path

lib_path = Path("src-tauri/src/lib.rs")
if not lib_path.exists():
    print("❌ lib.rs не найден")
    exit(1)

content = lib_path.read_text()

# Ищем блок с sidecar (с учётом возможных пробелов и переносов)
pattern = r'(#\[cfg\(target_os = "android"\)\]\s*\{\s*let handle = app\.handle\(\)\.clone\(\);\s*let Ok\(sidecar_command\) = handle\.shell\(\)\.sidecar\("sidecar"\) else \{ return Ok\(\(\)\); \};\s*let _ = sidecar_command\.spawn\(\)\.ok\(\);\s*\})'

replacement = '''#[cfg(target_os = "android")]
                {
                    let handle = app.handle().clone();
                    let domain = "cloudcode-pa.googleapis.com";
                    let ip = match crate::utils::doh::resolve_hostname(domain).await {
                        Ok(ip) => ip,
                        Err(e) => {
                            eprintln!("DoH resolve failed for {}: {}, using system DNS", domain, e);
                            domain.to_string()
                        }
                    };
                    let Ok(sidecar_cmd) = handle.shell().sidecar("sidecar") else { return Ok(()); };
                    let args = vec!["--host", domain, "--ip", &ip];
                    let _ = sidecar_cmd.args(args).spawn().ok();
                }'''

new_content, count = re.subn(pattern, replacement, content, flags=re.DOTALL)
if count:
    lib_path.write_text(new_content)
    print("✅ lib.rs успешно обновлён")
else:
    print("⚠️ Блок не найден, возможно уже изменён или имеет другой формат.")
    print("Пожалуйста, покажите содержимое lib.rs в районе .setup()")
