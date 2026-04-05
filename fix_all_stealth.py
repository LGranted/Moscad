#!/usr/bin/env python3
import re
import shutil
import subprocess
import sys
from pathlib import Path

# ---------- 1. Замена Go sidecar ----------
GO_NEW_CONTENT = '''package main

import (
    "bytes"
    "encoding/binary"
    "flag"
    "fmt"
    "io"
    "log"
    "net"
    "net/http"
    "os"
    "strings"
    "sync"
    "syscall"
    "time"

    utls "github.com/refraction-networking/utls"
    "golang.org/x/net/http2"
)

const (
    h2PrefaceStr     = "PRI * HTTP/2.0\\r\\n\\r\\nSM\\r\\n\\r\\n"
    h2PrefaceLen     = 24
    h2FrameHeaderLen = 9
    chromeWindowInc  = uint32(15663105)
)

type h2Setting struct {
    id  uint16
    val uint32
}

var chromeSettings = []h2Setting{
    {1, 65536},
    {2, 0},
    {4, 6291456},
    {6, 262144},
}

var (
    targetHost = flag.String("host", "", "Target hostname (for Host header)")
    targetIP   = flag.String("ip", "", "Target IP address (optional, if empty will resolve via system DNS)")
    sockPath   = flag.String("socket", "/data/data/com.lbjlaq.antigravity/files/utls.sock", "Unix socket path")
)

func setTCPOptions(conn *net.TCPConn) error {
    raw, err := conn.SyscallConn()
    if err != nil {
        return err
    }
    var ctrlErr error
    err = raw.Control(func(fd uintptr) {
        if err := syscall.SetsockoptInt(int(fd), syscall.IPPROTO_TCP, syscall.TCP_MAXSEG, 1460); err != nil {
            ctrlErr = err
            return
        }
        if err := syscall.SetsockoptInt(int(fd), syscall.IPPROTO_TCP, syscall.TCP_NODELAY, 1); err != nil {
            // non-critical
        }
        if err := syscall.SetsockoptInt(int(fd), syscall.SOL_SOCKET, syscall.SO_RCVBUF, 131072); err != nil {
            // non-critical
        }
    })
    if ctrlErr != nil {
        return ctrlErr
    }
    return err
}

func dialChrome(addr string) (net.Conn, error) {
    host, _, _ := net.SplitHostPort(addr)
    rawConn, err := net.DialTimeout("tcp", addr, 10*time.Second)
    if err != nil {
        return nil, err
    }
    if tcpConn, ok := rawConn.(*net.TCPConn); ok {
        if err := setTCPOptions(tcpConn); err != nil {
            log.Printf("Warning: failed to set TCP options: %v", err)
        }
    }
    uConn := utls.UClient(rawConn, &utls.Config{ServerName: host}, utls.HelloChrome_Auto)
    if err := uConn.Handshake(); err != nil {
        rawConn.Close()
        return nil, err
    }
    return &chromeConn{Conn: uConn}, nil
}

type chromeConn struct {
    net.Conn
    once sync.Once
}

func (c *chromeConn) Write(b []byte) (int, error) {
    var err error
    c.once.Do(func() {
        if strings.HasPrefix(string(b), h2PrefaceStr) {
            var buf bytes.Buffer
            buf.WriteString(h2PrefaceStr)
            buf.Write(buildSettingsFrame(chromeSettings))
            buf.Write(buildWindowUpdateFrame(0, chromeWindowInc))
            _, err = c.Conn.Write(buf.Bytes())
            b = b[h2PrefaceLen:]
        }
    })
    if err != nil {
        return 0, err
    }
    return c.Conn.Write(b)
}

func buildSettingsFrame(ss []h2Setting) []byte {
    payload := make([]byte, 6*len(ss))
    for i, s := range ss {
        binary.BigEndian.PutUint16(payload[i*6:], s.id)
        binary.BigEndian.PutUint32(payload[i*6+2:], s.val)
    }
    f := make([]byte, h2FrameHeaderLen+len(payload))
    f[0], f[1], f[2] = byte(len(payload)>>16), byte(len(payload)>>8), byte(len(payload))
    f[3] = 0x4
    copy(f[h2FrameHeaderLen:], payload)
    return f
}

func buildWindowUpdateFrame(streamID, inc uint32) []byte {
    f := make([]byte, h2FrameHeaderLen+4)
    f[0], f[1], f[2] = 0, 0, 4
    f[3] = 0x8
    binary.BigEndian.PutUint32(f[5:], streamID&0x7FFFFFFF)
    binary.BigEndian.PutUint32(f[9:], inc&0x7FFFFFFF)
    return f
}

func main() {
    flag.Parse()
    if *targetHost == "" {
        log.Fatal("--host is required")
    }
    dialAddr := *targetHost + ":443"
    if *targetIP != "" {
        dialAddr = *targetIP + ":443"
    }

    tr := &http2.Transport{
        DialTLS: func(network, addr string, cfg *utls.Config) (net.Conn, error) {
            return dialChrome(dialAddr)
        },
    }

    os.Remove(*sockPath)
    listener, err := net.Listen("unix", *sockPath)
    if err != nil {
        log.Fatalf("Listen error: %v", err)
    }
    defer listener.Close()
    os.Chmod(*sockPath, 0777)

    server := &http.Server{
        Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            targetURL := fmt.Sprintf("https://%s%s", *targetHost, r.URL.RequestURI())
            outReq, _ := http.NewRequest(r.Method, targetURL, r.Body)
            for k, vv := range r.Header {
                for _, v := range vv {
                    outReq.Header.Add(k, v)
                }
            }
            outReq.Host = *targetHost
            resp, err := tr.RoundTrip(outReq)
            if err != nil {
                http.Error(w, err.Error(), 502)
                return
            }
            defer resp.Body.Close()
            for k, vv := range resp.Header {
                for _, v := range vv {
                    w.Header().Add(k, v)
                }
            }
            w.WriteHeader(resp.StatusCode)
            io.Copy(w, resp.Body)
        }),
    }

    log.Printf("[sidecar] Chrome 133 Engine ready on %s", *sockPath)
    server.Serve(listener)
}
'''

def replace_go_sidecar():
    go_path = Path("sidecar-utls/main.go")
    if not go_path.exists():
        print("❌ sidecar-utls/main.go не найден")
        return False
    backup = go_path.with_suffix(".go.bak")
    if not backup.exists():
        shutil.copy(go_path, backup)
        print(f"✅ Создана резервная копия: {backup}")
    go_path.write_text(GO_NEW_CONTENT)
    print("✅ Go sidecar обновлён (флаги, TCP-опции, синхронизация DNS)")
    return True

# ---------- 2. Исправление lib.rs ----------
def fix_lib_rs():
    lib_path = Path("src-tauri/src/lib.rs")
    if not lib_path.exists():
        print("❌ lib.rs не найден")
        return False
    content = lib_path.read_text()
    # Ищем существующий блок запуска sidecar
    pattern = r'(\[#cfg\(target_os = "android"\)\]\s*\{\s*let handle = app\.handle\(\)\.clone\(\);\s*let Ok\(sidecar_command\) = handle\.shell\(\)\.sidecar\("sidecar"\) else \{ return Ok\(\(\)\); \};\s*let _ = sidecar_command\.spawn\(\)\.ok\(\);\s*\})'
    replacement = '''#[cfg(target_os = "android")]
                {
                    let handle = app.handle().clone();
                    // Resolve domain via DoH before starting sidecar
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
    new_content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    if new_content != content:
        lib_path.write_text(new_content)
        print("✅ lib.rs обновлён: sidecar запускается с DoH-IP")
        return True
    else:
        print("⚠️ Не удалось найти блок запуска sidecar в lib.rs. Возможно, он уже изменён или имеет другую структуру.")
        print("   Рекомендуется проверить вручную.")
        return False

# ---------- 3. Замена оставшихся Client::new ----------
def fix_remaining_clients():
    replacements = [
        ("src-tauri/src/commands/proxy.rs", r"reqwest::Client::builder\(\)", "crate::utils::http::get_client()"),
        ("src-tauri/src/proxy/proxy_pool.rs", r"Client::builder\(\)", "crate::utils::http::get_client()"),
        ("src-tauri/src/proxy/server.rs", r"reqwest::Client::new\(\)", "crate::utils::http::get_client()"),
        ("src-tauri/src/proxy/upstream/client.rs", r"Client::new\(\)", "crate::utils::http::get_client()"),
    ]
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

# ---------- 4. Проверка пути сокета ----------
def check_socket_path():
    rust_path = None
    for line in Path("src-tauri/src/utils/http.rs").read_text().splitlines():
        if "UTLS_SOCK_PATH" in line and "str" in line:
            parts = line.split('"')
            if len(parts) >= 2:
                rust_path = parts[1]
                break
    go_path = None
    for line in Path("sidecar-utls/main.go").read_text().splitlines():
        if 'sockPath' in line and 'String(' in line:
            parts = line.split('"')
            if len(parts) >= 2:
                go_path = parts[1]
                break
    if rust_path and go_path:
        if rust_path == go_path:
            print(f"✅ Путь сокета совпадает: {rust_path}")
        else:
            print(f"⚠️ Путь сокета не совпадает: Rust={rust_path}, Go={go_path}")
    else:
        print("❌ Не удалось определить пути сокета")

# ---------- Главная функция ----------
def main():
    print("=== Полное исправление стелс-слоя ===\n")
    ok = True
    if not replace_go_sidecar():
        ok = False
    if not fix_lib_rs():
        ok = False
    fix_remaining_clients()
    check_socket_path()
    print("\n=== Действия после скрипта ===")
    print("1. Соберите Go sidecar:")
    print("   cd sidecar-utls")
    print("   CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build -o moscad-sidecar main.go")
    print("   cp moscad-sidecar ../src-tauri/binaries/")
    print("2. Соберите APK:")
    print("   npx tauri android build")
    print("3. Проверьте логи sidecar на устройстве (logcat | grep sidecar)")
    if not ok:
        print("\n⚠️ Были проблемы, рекомендуется вручную проверить изменения.")

if __name__ == "__main__":
    main()
