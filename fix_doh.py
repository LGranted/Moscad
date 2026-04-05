#!/usr/bin/env python3
import re
import subprocess
import sys
from pathlib import Path

def modify_http_rs():
    path = Path("src-tauri/src/utils/http.rs")
    if not path.exists():
        print("❌ http.rs not found")
        return False
    content = path.read_text()
    
    # 1. Добавить DoH-резолвер после импортов (сразу после "use std::time::Duration;")
    marker = "use std::time::Duration;"
    doh_code = '''
// DoH resolver for Android using hickory-resolver
#[cfg(target_os = "android")]
mod doh_resolver {
    use hickory_resolver::{
        config::{ResolverConfig, ResolverOpts},
        name_server::TokioConnectionProvider,
        TokioResolver,
    };
    use reqwest::dns::{Resolve, Resolving};
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct DoHResolver {
        resolver: TokioResolver,
    }

    impl DoHResolver {
        pub fn new() -> Self {
            let resolver = TokioResolver::builder_with_config(
                ResolverConfig::cloudflare(),
                TokioConnectionProvider::default(),
            )
            .with_options(ResolverOpts::default())
            .build();
            Self { resolver }
        }
    }

    impl Resolve for DoHResolver {
        fn resolve(&self, name: reqwest::dns::Name) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<SocketAddr>>> + Send + '_>> {
            let resolver = self.resolver.clone();
            let name_str = name.as_str().to_string();
            Box::pin(async move {
                let response = resolver.lookup_ip(&name_str).await
                    .map_err(|e| anyhow::anyhow!("DoH lookup failed for {}: {}", name_str, e))?;
                let addrs: Vec<SocketAddr> = response.iter()
                    .map(|ip| SocketAddr::new(ip, 0))
                    .collect();
                if addrs.is_empty() {
                    Err(anyhow::anyhow!("No IP resolved for {}", name_str))
                } else {
                    Ok(addrs)
                }
            })
        }
    }
}
'''
    if "mod doh_resolver" not in content:
        content = content.replace(marker, marker + doh_code)
        print("✅ Добавлен DoH-резолвер в http.rs")
    else:
        print("ℹ️ DoH-резолвер уже присутствует в http.rs")
    
    # 2. Заменить create_base_client на версию с DoH для Android
    # Найдем текущую функцию
    func_start = content.find("fn create_base_client")
    if func_start == -1:
        print("❌ Не найдена create_base_client")
        return False
    # Найдем конец функции (следующий '}' на том же уровне)
    brace_count = 0
    end = func_start
    while end < len(content):
        if content[end] == '{':
            brace_count += 1
        elif content[end] == '}':
            brace_count -= 1
            if brace_count == 0:
                break
        end += 1
    old_func = content[func_start:end+1]
    
    new_func = '''fn create_base_client(timeout_secs: u64) -> Client {
    let builder = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .local_address(Some(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)))
        .user_agent(crate::utils::fingerprint::FingerprintConfig::current().user_agent.clone());
    #[cfg(target_os = "android")]
    let builder = builder.dns_resolver(doh_resolver::DoHResolver::new());
    #[cfg(not(target_os = "android"))]
    let builder = {
        if let Ok(config) = crate::modules::config::load_app_config() {
            let proxy_cfg = config.proxy.upstream_proxy;
            if proxy_cfg.enabled && !proxy_cfg.url.is_empty() {
                match reqwest::Proxy::all(&proxy_cfg.url) {
                    Ok(proxy) => builder.proxy(proxy),
                    Err(e) => {
                        tracing::error!("Invalid proxy URL '{}': {}", proxy_cfg.url, e);
                        builder
                    }
                }
            } else {
                builder
            }
        } else {
            builder
        }
    };
    builder.build().unwrap_or_else(|_| Client::new())
}'''
    
    if old_func != new_func:
        content = content.replace(old_func, new_func)
        print("✅ create_base_client переписана с поддержкой DoH")
    else:
        print("ℹ️ create_base_client уже имеет правильный вид")
    
    path.write_text(content)
    return True

def replace_client_new_with_global():
    # Файлы, где безопасно заменить reqwest::Client::new() на get_client()
    # Список получен из grep
    replacements = [
        ("src-tauri/src/proxy/handlers/claude.rs", r"reqwest::Client::new\(\)", "crate::utils::http::get_client()"),
        ("src-tauri/src/proxy/server.rs", r"let client = reqwest::Client::new\(\);", "let client = crate::utils::http::get_client();"),
        ("src-tauri/src/proxy/upstream/client.rs", r"Client::new\(\)", "crate::utils::http::get_client()"),
        ("src-tauri/src/android_stubs.rs", r"let client = reqwest::Client::new\(\);", "let client = crate::utils::http::get_client();"),
    ]
    success = True
    for file, pattern, replacement in replacements:
        path = Path(file)
        if not path.exists():
            print(f"⚠️ {file} не найден, пропускаем")
            continue
        content = path.read_text()
        new_content = re.sub(pattern, replacement, content)
        if new_content != content:
            path.write_text(new_content)
            print(f"✅ Заменено в {file}")
        else:
            print(f"ℹ️ В {file} ничего не заменено (возможно уже исправлено)")
    return success

def check_changes():
    # Проверка: ищем вызовы reqwest::Client::new (должно остаться только в create_base_client и в блокирующих)
    print("\n=== Проверка изменений ===")
    result = subprocess.run(["grep", "-rn", "reqwest::Client::new", "src-tauri/src/", "--include=*.rs"], capture_output=True, text=True)
    if result.stdout:
        print("⚠️ Найдены прямые вызовы reqwest::Client::new (кроме create_base_client):")
        for line in result.stdout.splitlines():
            if "create_base_client" not in line:
                print(f"   {line}")
    else:
        print("✅ Прямых вызовов reqwest::Client::new не осталось (кроме create_base_client)")
    
    # Проверка наличия doh_resolver
    result = subprocess.run(["grep", "-n", "doh_resolver::DoHResolver", "src-tauri/src/utils/http.rs"], capture_output=True, text=True)
    if result.stdout:
        print("✅ DoH-резолвер успешно добавлен")
    else:
        print("❌ DoH-резолвер не найден в http.rs")
    
    # Проверка компиляции (опционально)
    print("\n=== Компиляция (проверка синтаксиса) ===")
    result = subprocess.run(["cargo", "check", "--target", "aarch64-linux-android", "--message-format=short"], capture_output=True, text=True)
    if result.returncode == 0:
        print("✅ Код успешно компилируется")
    else:
        print("❌ Ошибки компиляции. Вывод:")
        print(result.stderr[:1000])

def main():
    print("Начинаем автоматическое исправление DoH...")
    if not modify_http_rs():
        sys.exit(1)
    if not replace_client_new_with_global():
        sys.exit(1)
    check_changes()
    print("\nГотово! Рекомендуется проверить сборку APK.")

if __name__ == "__main__":
    main()
