#!/usr/bin/env bash
# install_sidecar.sh
# ─────────────────────────────────────────────────────────────────────────────
# Запусти один раз в Termux из корня проекта Moscad:
#   chmod +x install_sidecar.sh && ./install_sidecar.sh
#
# Что делает скрипт:
#   1. Устанавливает Go и зависимости Termux
#   2. Создаёт sidecar-utls/main.go + go.mod
#   3. Компилирует под aarch64-linux-android (native Termux build)
#   4. Кладёт бинарник в src-tauri/binaries/
#   5. Заменяет src-tauri/src/utils/http.rs на UDS-версию
#   6. Заменяет src-tauri/src/lib.rs на версию с supervisor'ом
#   7. Прописывает externalBin в tauri.conf.json
#   8. Добавляет зависимости в Cargo.toml (tower, tokio UnixStream)
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

# ── Цвета ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
ok()      { echo -e "${GREEN}[OK]${RESET}    $*"; }
warn()    { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
die()     { echo -e "${RED}[ERROR]${RESET} $*" >&2; exit 1; }

# ── Проверка рабочей директории ───────────────────────────────────────────────
[[ -f "src-tauri/tauri.conf.json" ]] || \
    die "Запусти скрипт из корня проекта Moscad (там должен быть src-tauri/tauri.conf.json)"

PROJECT_ROOT="$(pwd)"
SIDECAR_DIR="$PROJECT_ROOT/sidecar-utls"
BINARIES_DIR="$PROJECT_ROOT/src-tauri/binaries"
SIDECAR_BIN="$BINARIES_DIR/sidecar-aarch64-linux-android"
TAURI_CONF="$PROJECT_ROOT/src-tauri/tauri.conf.json"
CARGO_TOML="$PROJECT_ROOT/src-tauri/Cargo.toml"
HTTP_RS="$PROJECT_ROOT/src-tauri/src/utils/http.rs"
LIB_RS="$PROJECT_ROOT/src-tauri/src/lib.rs"

echo -e "\n${CYAN}═══════════════════════════════════════════════════════${RESET}"
echo -e "${CYAN}  Moscad uTLS Sidecar Installer (Chrome 131 / Android)  ${RESET}"
echo -e "${CYAN}═══════════════════════════════════════════════════════${RESET}\n"

# ─────────────────────────────────────────────────────────────────────────────
# ШАГ 1 — Установка Go
# ─────────────────────────────────────────────────────────────────────────────
step1_install_go() {
    info "Шаг 1/7 → Проверка/установка Go"
    if ! command -v go &>/dev/null; then
        info "Go не найден, устанавливаю через pkg..."
        pkg update -y && pkg install -y golang
    fi
    local go_ver
    go_ver="$(go version 2>&1)"
    ok "Go: $go_ver"
}

# ─────────────────────────────────────────────────────────────────────────────
# ШАГ 2 — Создание sidecar-utls/main.go
# ─────────────────────────────────────────────────────────────────────────────
step2_write_go_source() {
    info "Шаг 2/7 → Запись Go-исходников"
    mkdir -p "$SIDECAR_DIR"

    # ── go.mod ────────────────────────────────────────────────────────────────
    cat > "$SIDECAR_DIR/go.mod" << 'GOMOD'
module moscad-sidecar

go 1.21

require (
	github.com/refraction-networking/utls v1.6.7
	golang.org/x/net v0.27.0
)

require (
	github.com/andybalholm/brotli v1.1.0 // indirect
	github.com/cloudflare/circl v1.3.9 // indirect
	github.com/klauspost/compress v1.17.9 // indirect
	golang.org/x/crypto v0.25.0 // indirect
	golang.org/x/sys v0.22.0 // indirect
	golang.org/x/text v0.16.0 // indirect
)
GOMOD

    # ── main.go ───────────────────────────────────────────────────────────────
    cat > "$SIDECAR_DIR/main.go" << 'GOMAINEOF'
// sidecar-utls/main.go
// Chrome 131 TLS + HTTP/2 proxy over Unix Domain Socket
// Rust → UDS (HTTP/1.1) → [sidecar] → googleapis (uTLS/H2)
package main

import (
	"bufio"
	"bytes"
	"encoding/binary"
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"strings"
	"sync"
	"syscall"
	"time"

	utls "github.com/refraction-networking/utls"
	"golang.org/x/net/http2"
)

const (
	defaultSockPath              = "/data/data/com.lbjlaq.antigravity_tools/files/utls.sock"
	chrome131ConnWindowIncrement = uint32(15663105) // 15728640 − 65535
	h2PrefaceStr                 = "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
	h2PrefaceLen                 = 24
	h2FrameHeaderLen             = 9
)

// Chrome 131 SETTINGS: HEADER_TABLE_SIZE=65536, ENABLE_PUSH=0,
// INITIAL_WINDOW_SIZE=6291456, MAX_HEADER_LIST_SIZE=262144
type h2Setting struct{ id uint16; val uint32 }

var chrome131Settings = []h2Setting{
	{0x1, 65536}, {0x2, 0}, {0x4, 6291456}, {0x6, 262144},
}

var sockPath string

func main() {
	flag.StringVar(&sockPath, "socket", defaultSockPath, "Unix Domain Socket path")
	flag.Parse()

	os.Remove(sockPath)
	ln, err := net.Listen("unix", sockPath)
	if err != nil {
		log.Fatalf("[sidecar] Listen %s: %v", sockPath, err)
	}
	defer ln.Close()
	os.Chmod(sockPath, 0600)
	log.Printf("[sidecar] uTLS/Chrome_131 H2 proxy ready on %s", sockPath)

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGTERM, syscall.SIGINT)
	go func() {
		<-sigCh
		ln.Close()
		os.Remove(sockPath)
		os.Exit(0)
	}()

	for {
		conn, err := ln.Accept()
		if err != nil {
			if strings.Contains(err.Error(), "use of closed network connection") {
				return
			}
			log.Printf("[sidecar] Accept: %v", err)
			continue
		}
		go handleConn(conn)
	}
}

// ── Per-host pooled HTTP/2 transports ────────────────────────────────────────

var (
	transportsMu sync.Mutex
	transports   = make(map[string]*http2.Transport)
)

func getTransport(hostport string) *http2.Transport {
	transportsMu.Lock()
	defer transportsMu.Unlock()
	if t, ok := transports[hostport]; ok {
		return t
	}
	hp := hostport
	t := &http2.Transport{
		DialTLS: func(network, addr string, _ interface{ Clone() interface{} }) (net.Conn, error) {
			return dialChrome131(hp)
		},
	}
	transports[hostport] = t
	return t
}

// ── uTLS dialer ───────────────────────────────────────────────────────────────

func dialChrome131(addr string) (net.Conn, error) {
	host, port, err := net.SplitHostPort(addr)
	if err != nil {
		host, port = addr, "443"
	}
	rawConn, err := net.DialTimeout("tcp", net.JoinHostPort(host, port), 15*time.Second)
	if err != nil {
		return nil, fmt.Errorf("tcp %s: %w", addr, err)
	}
	if tc, ok := rawConn.(*net.TCPConn); ok {
		tc.SetKeepAlive(true)
		tc.SetKeepAlivePeriod(60 * time.Second)
	}

	tlsConn := utls.UClient(rawConn, &utls.Config{ServerName: host}, utls.HelloChrome_131)
	if err := tlsConn.Handshake(); err != nil {
		rawConn.Close()
		return nil, fmt.Errorf("utls handshake %s: %w", host, err)
	}
	if tlsConn.ConnectionState().NegotiatedProtocol != "h2" {
		tlsConn.Close()
		return nil, fmt.Errorf("%s: no h2 negotiated", host)
	}
	return &chrome131Conn{Conn: tlsConn}, nil
}

// ── chrome131Conn: intercepts & replaces H2 SETTINGS ─────────────────────────

type chrome131Conn struct {
	net.Conn
	mu       sync.Mutex
	buf      bytes.Buffer
	injected bool
}

func (c *chrome131Conn) Write(b []byte) (int, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.injected {
		return c.Conn.Write(b)
	}
	c.buf.Write(b)
	data := c.buf.Bytes()
	if len(data) < h2PrefaceLen+h2FrameHeaderLen {
		return len(b), nil
	}
	if string(data[:h2PrefaceLen]) != h2PrefaceStr {
		c.injected = true
		out := make([]byte, len(data))
		copy(out, data)
		c.buf.Reset()
		_, err := c.Conn.Write(out)
		return len(b), err
	}
	sh := data[h2PrefaceLen : h2PrefaceLen+h2FrameHeaderLen]
	payloadLen := int(sh[0])<<16 | int(sh[1])<<8 | int(sh[2])
	totalOrig := h2PrefaceLen + h2FrameHeaderLen + payloadLen
	if len(data) < totalOrig {
		return len(b), nil // buffer more
	}
	tail := make([]byte, len(data)-totalOrig)
	copy(tail, data[totalOrig:])
	c.buf.Reset()
	c.injected = true

	var out bytes.Buffer
	out.WriteString(h2PrefaceStr)
	out.Write(buildSettingsFrame(chrome131Settings))
	out.Write(buildWindowUpdateFrame(0, chrome131ConnWindowIncrement))
	out.Write(tail)
	_, err := c.Conn.Write(out.Bytes())
	return len(b), err
}

func buildSettingsFrame(ss []h2Setting) []byte {
	payload := make([]byte, 6*len(ss))
	for i, s := range ss {
		binary.BigEndian.PutUint16(payload[i*6:], s.id)
		binary.BigEndian.PutUint32(payload[i*6+2:], s.val)
	}
	pLen := len(payload)
	frame := make([]byte, h2FrameHeaderLen+pLen)
	frame[0], frame[1], frame[2] = byte(pLen>>16), byte(pLen>>8), byte(pLen)
	frame[3] = 0x4 // SETTINGS type
	copy(frame[h2FrameHeaderLen:], payload)
	return frame
}

func buildWindowUpdateFrame(streamID, inc uint32) []byte {
	f := make([]byte, h2FrameHeaderLen+4)
	f[0], f[1], f[2] = 0, 0, 4; f[3] = 0x8 // WINDOW_UPDATE
	binary.BigEndian.PutUint32(f[5:], streamID&0x7FFFFFFF)
	binary.BigEndian.PutUint32(f[9:], inc&0x7FFFFFFF)
	return f
}

// ── UDS request handler ───────────────────────────────────────────────────────

func handleConn(clientConn net.Conn) {
	defer clientConn.Close()
	clientConn.SetDeadline(time.Now().Add(120 * time.Second))

	req, err := http.ReadRequest(bufio.NewReaderSize(clientConn, 64*1024))
	if err != nil {
		return
	}
	defer req.Body.Close()

	targetHost := req.Host
	if targetHost == "" && req.URL != nil {
		targetHost = req.URL.Host
	}
	if targetHost == "" {
		writeError(clientConn, 400, "missing Host"); return
	}
	hostport := targetHost
	if !strings.Contains(hostport, ":") {
		hostport += ":443"
	}

	upURL := &url.URL{Scheme: "https", Host: targetHost,
		Path: req.URL.Path, RawQuery: req.URL.RawQuery}
	outReq, err := http.NewRequest(req.Method, upURL.String(), req.Body)
	if err != nil {
		writeError(clientConn, 500, err.Error()); return
	}
	for k, vv := range req.Header {
		switch strings.ToLower(k) {
		case "proxy-connection", "proxy-authorization",
			"te", "trailers", "transfer-encoding", "upgrade":
			continue
		}
		for _, v := range vv {
			outReq.Header.Add(k, v)
		}
	}
	outReq.Host = targetHost

	resp, err := getTransport(hostport).RoundTrip(outReq)
	if err != nil {
		log.Printf("[sidecar] RoundTrip %s: %v", upURL, err)
		writeError(clientConn, 502, err.Error()); return
	}
	defer resp.Body.Close()

	// Stream response — no buffering (critical for SSE)
	st := http.StatusText(resp.StatusCode)
	if st == "" { st = "Unknown" }
	fmt.Fprintf(clientConn, "HTTP/1.1 %d %s\r\n", resp.StatusCode, st)
	for k, vv := range resp.Header {
		for _, v := range vv {
			fmt.Fprintf(clientConn, "%s: %s\r\n", k, v)
		}
	}
	fmt.Fprintf(clientConn, "\r\n")
	io.Copy(clientConn, resp.Body)
}

func writeError(w io.Writer, code int, msg string) {
	body := fmt.Sprintf(`{"error":%q}`, msg)
	fmt.Fprintf(w, "HTTP/1.1 %d %s\r\nContent-Type: application/json\r\nContent-Length: %d\r\n\r\n%s",
		code, http.StatusText(code), len(body), body)
}
GOMAINEOF

    ok "Go-исходники записаны"
}

# ─────────────────────────────────────────────────────────────────────────────
# ШАГ 3 — Сборка Go-бинарника
# ─────────────────────────────────────────────────────────────────────────────
step3_build_go() {
    info "Шаг 3/7 → Компиляция Go сайдкара (native aarch64)"
    mkdir -p "$BINARIES_DIR"
    cd "$SIDECAR_DIR"

    info "Скачивание Go-зависимостей..."
    go mod tidy

    info "Сборка бинарника..."
    CGO_ENABLED=0 go build \
        -ldflags="-s -w" \
        -trimpath \
        -o "$SIDECAR_BIN" \
        .

    chmod +x "$SIDECAR_BIN"
    cd "$PROJECT_ROOT"

    local size
    size=$(du -sh "$SIDECAR_BIN" | cut -f1)
    ok "Бинарник: $SIDECAR_BIN ($size)"
}

# ─────────────────────────────────────────────────────────────────────────────
# ШАГ 4 — Обновление http.rs
# ─────────────────────────────────────────────────────────────────────────────
step4_patch_http_rs() {
    info "Шаг 4/7 → Замена $HTTP_RS"
    mkdir -p "$(dirname "$HTTP_RS")"
    # Скрипт сам создаёт http.rs выше; здесь копируем из выходного каталога,
    # если скрипт запущен после того, как файлы уже написаны вручную.
    # На случай standalone-запуска — пишем inline.
    cat > "$HTTP_RS" << 'HTTPRSEOF'
// src-tauri/src/utils/http.rs  [auto-generated by install_sidecar.sh]
// Stealth module routes ALL googleapis traffic through Go uTLS sidecar via UDS.

#[cfg(not(target_os = "android"))]
use crate::modules::config::load_app_config;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

pub const UTLS_SOCK_PATH: &str =
    "/data/data/com.lbjlaq.antigravity_tools/files/utls.sock";

pub static SHARED_CLIENT: Lazy<Client> = Lazy::new(|| create_base_client(15));
pub static SHARED_CLIENT_LONG: Lazy<Client> = Lazy::new(|| create_base_client(60));
pub static SHARED_STANDARD_CLIENT: Lazy<Client> = Lazy::new(|| create_base_client(15));
pub static SHARED_STANDARD_CLIENT_LONG: Lazy<Client> = Lazy::new(|| create_base_client(60));

fn create_base_client(timeout_secs: u64) -> Client {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .user_agent(
            crate::utils::fingerprint::FingerprintConfig::current()
                .user_agent
                .clone(),
        );

    #[cfg(not(target_os = "android"))]
    if let Ok(config) = load_app_config() {
        let proxy_cfg = config.proxy.upstream_proxy;
        if proxy_cfg.enabled && !proxy_cfg.url.is_empty() {
            match reqwest::Proxy::all(&proxy_cfg.url) {
                Ok(proxy) => { builder = builder.proxy(proxy); }
                Err(e) => { tracing::error!("Invalid proxy URL: {}", e); }
            }
        }
    }

    builder.build().unwrap_or_else(|_| Client::new())
}

pub fn get_client() -> Client { SHARED_CLIENT.clone() }
pub fn get_long_client() -> Client { SHARED_CLIENT_LONG.clone() }
pub fn get_standard_client() -> Client { SHARED_STANDARD_CLIENT.clone() }
pub fn get_long_standard_client() -> Client { SHARED_STANDARD_CLIENT_LONG.clone() }

#[cfg(target_os = "android")]
pub mod stealth {
    use super::UTLS_SOCK_PATH;
    use hyper014::client::connect::{Connected, Connection};
    use hyper014::Uri;
    use std::future::Future;
    use std::io;
    use std::path::PathBuf;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::net::UnixStream;

    pub type StealthClient = hyper014::Client<UnixConnector, hyper014::Body>;

    pub fn get_stealth_client() -> anyhow::Result<StealthClient> {
        get_stealth_client_for(None)
    }
    pub fn get_stealth_client_for_account(s: Option<&str>) -> anyhow::Result<StealthClient> {
        get_stealth_client_for(s)
    }
    pub fn get_stealth_client_for(_account_seed: Option<&str>) -> anyhow::Result<StealthClient> {
        let connector = UnixConnector::new(UTLS_SOCK_PATH);
        let client = hyper014::Client::builder()
            .http1_only(true)
            .pool_max_idle_per_host(8)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .retry_canceled_requests(true)
            .build::<_, hyper014::Body>(connector);
        tracing::info!("[stealth] UDS client → {} (Chrome 131 sidecar)", UTLS_SOCK_PATH);
        Ok(client)
    }

    #[derive(Clone, Debug)]
    pub struct UnixConnector { path: PathBuf }
    impl UnixConnector {
        pub fn new(path: impl Into<PathBuf>) -> Self { Self { path: path.into() } }
    }
    impl tower::Service<Uri> for UnixConnector {
        type Response = UnixStream;
        type Error = io::Error;
        type Future = Pin<Box<dyn Future<Output = io::Result<UnixStream>> + Send + 'static>>;
        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, _: Uri) -> Self::Future {
            let path = self.path.clone();
            Box::pin(async move { UnixStream::connect(path).await })
        }
    }
    impl Connection for UnixStream {
        fn connected(&self) -> Connected { Connected::new() }
    }

    /// Build a hyper request with all Moscad-specific headers.
    /// Use `http://` scheme — TLS is handled by the sidecar.
    pub fn build_request(
        method: hyper014::Method,
        uri: &str,
        authorization: &str,
        machine_id: Option<&str>,
        body: bytes::Bytes,
    ) -> anyhow::Result<hyper014::Request<hyper014::Body>> {
        use hyper014::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE, HOST};
        let parsed: Uri = uri.parse()?;
        let host = parsed.authority().map(|a| a.as_str().to_string()).unwrap_or_default();
        let fp = crate::utils::fingerprint::FingerprintConfig::current();
        let mut req = hyper014::Request::builder()
            .method(method).uri(&parsed)
            .header(HOST, HeaderValue::from_str(&host)?)
            .header(AUTHORIZATION, HeaderValue::from_str(authorization)?)
            .header(CONTENT_TYPE, "application/json")
            .header("x-goog-api-client",
                format!("gl-node/22.18.0 antigravity/{}", fp.antigravity_version))
            .header("user-agent", &fp.user_agent);
        if let Some(mid) = machine_id {
            req = req.header("x-machine-id", mid);
        }
        Ok(req.body(hyper014::Body::from(body))?)
    }
}
HTTPRSEOF
    ok "http.rs обновлён"
}

# ─────────────────────────────────────────────────────────────────────────────
# ШАГ 5 — Обновление lib.rs
# ─────────────────────────────────────────────────────────────────────────────
step5_patch_lib_rs() {
    info "Шаг 5/7 → Замена $LIB_RS"

    # Резервная копия оригинала
    [[ -f "$LIB_RS" ]] && cp "$LIB_RS" "${LIB_RS}.bak"

    cat > "$LIB_RS" << 'LIBRSEOF'
// src-tauri/src/lib.rs  [auto-generated by install_sidecar.sh]
use log::info;
use tauri::Emitter;

pub mod android_stubs;
pub mod utils;
pub mod commands;
#[cfg(target_os = "android")] pub mod modules;
#[cfg(target_os = "android")] pub use android_stubs::*;
#[cfg(target_os = "android")]
pub use commands::proxy_android_stub::{
    handle_android_stealth_request, handle_android_stealth_request_stream,
};

// ── Sidecar supervisor ────────────────────────────────────────────────────────
#[cfg(target_os = "android")]
mod sidecar {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::process::Command;

    pub const UTLS_SOCK_PATH: &str =
        "/data/data/com.lbjlaq.antigravity_tools/files/utls.sock";
    const SIDECAR_BIN: &str = "sidecar-aarch64-linux-android";
    static RUNNING: AtomicBool = AtomicBool::new(false);

    fn sidecar_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
        use tauri::Manager;
        let rd = app.path().resource_dir().ok()?;
        let p = rd.join("binaries").join(SIDECAR_BIN);
        if p.exists() { return Some(p); }
        let p2 = rd.join(SIDECAR_BIN);
        if p2.exists() { return Some(p2); }
        log::error!("[sidecar] Binary not found in {:?}", rd);
        None
    }

    fn ensure_executable(path: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(m) = std::fs::metadata(path) {
            let mut p = m.permissions(); p.set_mode(0o755);
            let _ = std::fs::set_permissions(path, p);
        }
    }

    pub async fn supervise(app: tauri::AppHandle, shutdown: Arc<AtomicBool>) {
        let mut backoff = Duration::from_millis(500);
        loop {
            if shutdown.load(Ordering::Relaxed) { return; }
            let Some(path) = sidecar_path(&app) else {
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
                continue;
            };
            ensure_executable(&path);
            let _ = std::fs::remove_file(UTLS_SOCK_PATH);
            match Command::new(&path).args(["--socket", UTLS_SOCK_PATH])
                .kill_on_drop(true).spawn()
            {
                Ok(mut child) => {
                    let pid = child.id().unwrap_or(0);
                    log::info!("[sidecar] PID {} started", pid);
                    RUNNING.store(true, Ordering::Relaxed);
                    backoff = Duration::from_millis(500);
                    // Wait for socket file (sidecar ready)
                    let dl = tokio::time::Instant::now() + Duration::from_secs(3);
                    while tokio::time::Instant::now() < dl {
                        if std::path::Path::new(UTLS_SOCK_PATH).exists() { break; }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    let _ = app.emit("sidecar-ready", pid);
                    let _ = child.wait().await;
                    RUNNING.store(false, Ordering::Relaxed);
                    log::warn!("[sidecar] Process exited, restarting...");
                }
                Err(e) => { log::error!("[sidecar] Spawn failed: {}", e); }
            }
            if shutdown.load(Ordering::Relaxed) { return; }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(30));
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String { format!("Hello, {}!", name) }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "android")]
    let shutdown_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    #[cfg(target_os = "android")]
    let shutdown_setup = shutdown_flag.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            #[cfg(target_os = "android")] get_device_profiles,
            #[cfg(target_os = "android")] bind_device_profile,
            #[cfg(target_os = "android")] bind_device_profile_with_profile,
            #[cfg(target_os = "android")] preview_generate_profile,
            #[cfg(target_os = "android")] apply_device_profile,
            #[cfg(target_os = "android")] start_proxy_foreground_service,
            #[cfg(target_os = "android")] start_proxy_service,
            #[cfg(target_os = "android")] stop_proxy_service,
            #[cfg(target_os = "android")] get_proxy_status,
            #[cfg(target_os = "android")] handle_android_stealth_request,
            #[cfg(target_os = "android")] list_accounts,
            #[cfg(target_os = "android")] get_current_account,
            #[cfg(target_os = "android")] add_account,
            #[cfg(target_os = "android")] delete_account,
            #[cfg(target_os = "android")] delete_accounts,
            #[cfg(target_os = "android")] switch_account,
            #[cfg(target_os = "android")] fetch_account_quota,
            #[cfg(target_os = "android")] refresh_account_quota,
            #[cfg(target_os = "android")] refresh_all_quotas,
            #[cfg(target_os = "android")] reorder_accounts,
            #[cfg(target_os = "android")] toggle_proxy_status,
            #[cfg(target_os = "android")] warm_up_all_accounts,
            #[cfg(target_os = "android")] warm_up_account,
            #[cfg(target_os = "android")] start_oauth_login,
            #[cfg(target_os = "android")] complete_oauth_login,
            #[cfg(target_os = "android")] cancel_oauth_login,
            #[cfg(target_os = "android")] prepare_oauth_url,
            #[cfg(target_os = "android")] submit_oauth_code,
            #[cfg(target_os = "android")] import_v1_accounts,
            #[cfg(target_os = "android")] import_from_db,
            #[cfg(target_os = "android")] import_custom_db,
            #[cfg(target_os = "android")] sync_account_from_db,
            #[cfg(target_os = "android")] list_device_versions,
            #[cfg(target_os = "android")] restore_device_version,
            #[cfg(target_os = "android")] delete_device_version,
            #[cfg(target_os = "android")] open_device_folder,
            #[cfg(target_os = "android")] restore_original_device,
            #[cfg(target_os = "android")] update_model_mapping,
            #[cfg(target_os = "android")] generate_api_key,
            #[cfg(target_os = "android")] clear_proxy_session_bindings,
            #[cfg(target_os = "android")] clear_proxy_rate_limit,
            #[cfg(target_os = "android")] clear_all_proxy_rate_limits,
            #[cfg(target_os = "android")] check_proxy_health,
            #[cfg(target_os = "android")] get_preferred_account,
            #[cfg(target_os = "android")] set_preferred_account,
            #[cfg(target_os = "android")] fetch_zai_models,
            #[cfg(target_os = "android")] load_config,
            #[cfg(target_os = "android")] save_config,
            #[cfg(target_os = "android")] get_proxy_stats,
            #[cfg(target_os = "android")] set_proxy_monitor_enabled,
            #[cfg(target_os = "android")] get_proxy_logs_filtered,
            #[cfg(target_os = "android")] get_proxy_logs_count_filtered,
            #[cfg(target_os = "android")] clear_proxy_logs,
            #[cfg(target_os = "android")] get_proxy_log_detail,
            #[cfg(target_os = "android")] enable_debug_console,
            #[cfg(target_os = "android")] disable_debug_console,
            #[cfg(target_os = "android")] is_debug_console_enabled,
            #[cfg(target_os = "android")] get_debug_console_logs,
            #[cfg(target_os = "android")] clear_debug_console_logs,
            #[cfg(target_os = "android")] get_cli_sync_status,
            #[cfg(target_os = "android")] execute_cli_sync,
            #[cfg(target_os = "android")] execute_cli_restore,
            #[cfg(target_os = "android")] get_cli_config_content,
            #[cfg(target_os = "android")] get_opencode_sync_status,
            #[cfg(target_os = "android")] execute_opencode_sync,
            #[cfg(target_os = "android")] execute_opencode_restore,
            #[cfg(target_os = "android")] execute_opencode_clear,
            #[cfg(target_os = "android")] get_opencode_config_content,
            #[cfg(target_os = "android")] get_token_stats_hourly,
            #[cfg(target_os = "android")] get_token_stats_daily,
            #[cfg(target_os = "android")] get_token_stats_weekly,
            #[cfg(target_os = "android")] get_token_stats_by_account,
            #[cfg(target_os = "android")] get_token_stats_summary,
            #[cfg(target_os = "android")] get_token_stats_by_model,
            #[cfg(target_os = "android")] get_token_stats_model_trend_hourly,
            #[cfg(target_os = "android")] get_token_stats_model_trend_daily,
            #[cfg(target_os = "android")] get_token_stats_account_trend_hourly,
            #[cfg(target_os = "android")] get_token_stats_account_trend_daily,
            #[cfg(target_os = "android")] clear_token_stats,
            #[cfg(target_os = "android")] get_data_dir_path,
            #[cfg(target_os = "android")] get_update_settings,
            #[cfg(target_os = "android")] save_update_settings,
            #[cfg(target_os = "android")] is_auto_launch_enabled,
            #[cfg(target_os = "android")] toggle_auto_launch,
            #[cfg(target_os = "android")] get_http_api_settings,
            #[cfg(target_os = "android")] save_http_api_settings,
            #[cfg(target_os = "android")] get_antigravity_path,
            #[cfg(target_os = "android")] get_antigravity_args,
            #[cfg(target_os = "android")] cloudflared_install,
            #[cfg(target_os = "android")] cloudflared_start,
            #[cfg(target_os = "android")] cloudflared_stop,
            #[cfg(target_os = "android")] cloudflared_get_status,
            #[cfg(target_os = "android")] should_check_updates,
            #[cfg(target_os = "android")] check_for_updates,
            #[cfg(target_os = "android")] update_last_check_time,
            #[cfg(target_os = "android")] get_ip_access_logs,
            #[cfg(target_os = "android")] clear_ip_access_logs,
            #[cfg(target_os = "android")] get_ip_stats,
            #[cfg(target_os = "android")] get_ip_token_stats,
            #[cfg(target_os = "android")] get_ip_blacklist,
            #[cfg(target_os = "android")] add_ip_to_blacklist,
            #[cfg(target_os = "android")] remove_ip_from_blacklist,
            #[cfg(target_os = "android")] clear_ip_blacklist,
            #[cfg(target_os = "android")] check_ip_in_blacklist,
            #[cfg(target_os = "android")] get_ip_whitelist,
            #[cfg(target_os = "android")] add_ip_to_whitelist,
            #[cfg(target_os = "android")] remove_ip_from_whitelist,
            #[cfg(target_os = "android")] clear_ip_whitelist,
            #[cfg(target_os = "android")] check_ip_in_whitelist,
            #[cfg(target_os = "android")] get_security_config,
            #[cfg(target_os = "android")] update_security_config,
            #[cfg(target_os = "android")] list_user_tokens,
            #[cfg(target_os = "android")] get_user_token_summary,
            #[cfg(target_os = "android")] create_user_token,
            #[cfg(target_os = "android")] renew_user_token,
            #[cfg(target_os = "android")] delete_user_token,
            #[cfg(target_os = "android")] update_user_token,
            #[cfg(target_os = "android")] get_proxy_pool_config,
            #[cfg(target_os = "android")] get_all_account_bindings,
            #[cfg(target_os = "android")] bind_account_proxy,
            #[cfg(target_os = "android")] unbind_account_proxy,
            #[cfg(target_os = "android")] get_account_proxy_binding,
            #[cfg(target_os = "android")] get_proxy_scheduling_config,
            #[cfg(target_os = "android")] update_proxy_scheduling_config,
            #[cfg(target_os = "android")] open_data_folder,
            #[cfg(target_os = "android")] clear_antigravity_cache,
            #[cfg(target_os = "android")] get_antigravity_cache_paths,
            #[cfg(target_os = "android")] clear_log_cache,
            #[cfg(target_os = "android")] update_account_label,
            #[cfg(target_os = "android")] export_accounts,
            #[cfg(target_os = "android")] reload_proxy_accounts,
            #[cfg(target_os = "android")] get_proxy_logs_paginated,
            #[cfg(target_os = "android")] get_proxy_logs_count,
            #[cfg(target_os = "android")] export_proxy_logs,
            #[cfg(target_os = "android")] export_proxy_logs_json,
            #[cfg(target_os = "android")] get_proxy_logs,
            #[cfg(target_os = "android")] handle_android_stealth_request_stream
        ])
        .setup(move |app| {
            #[cfg(target_os = "android")]
            {
                info!("[Moscad] Android Stealth Layer + uTLS sidecar starting");
                let _ = app.emit("android-ready", "Stealth mode active");
                let app_h = app.handle().clone();
                let sd = shutdown_setup.clone();
                tauri::async_runtime::spawn(async move {
                    crate::sidecar::supervise(app_h, sd).await;
                });
            }
            Ok(())
        })
        .on_window_event(move |_win, event| {
            #[cfg(target_os = "android")]
            if let tauri::WindowEvent::Destroyed = event {
                shutdown_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                let _ = std::fs::remove_file(crate::sidecar::UTLS_SOCK_PATH);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
LIBRSEOF
    ok "lib.rs обновлён"
}

# ─────────────────────────────────────────────────────────────────────────────
# ШАГ 6 — Cargo.toml: добавление зависимостей
# ─────────────────────────────────────────────────────────────────────────────
step6_patch_cargo_toml() {
    info "Шаг 6/7 → Проверка Cargo.toml"
    local needs_tower=0
    local needs_bytes=0

    grep -q 'tower' "$CARGO_TOML" || needs_tower=1
    grep -q '^bytes' "$CARGO_TOML" || needs_bytes=1

    if [[ $needs_tower -eq 1 ]] || [[ $needs_bytes -eq 1 ]]; then
        info "Добавляю недостающие зависимости в Cargo.toml..."
        # Найти секцию [dependencies] и добавить после неё
        if [[ $needs_tower -eq 1 ]]; then
            # Insert after [dependencies] line
            sed -i '/^\[dependencies\]/a tower = { version = "0.4", features = ["util"] }' "$CARGO_TOML"
            ok "tower добавлен"
        fi
        if [[ $needs_bytes -eq 1 ]]; then
            sed -i '/^\[dependencies\]/a bytes = "1"' "$CARGO_TOML"
            ok "bytes добавлен"
        fi
    else
        ok "Cargo.toml уже содержит нужные зависимости"
    fi

    # Убедиться, что hyper014 (hyper 0.14) присутствует с unix feature
    if grep -q 'hyper014\|"hyper"' "$CARGO_TOML"; then
        # Добавить unix socket feature если отсутствует
        if ! grep -q 'tokio.*"net"' "$CARGO_TOML" 2>/dev/null; then
            warn "Убедись, что в Cargo.toml у tokio есть features = [..., \"net\"] для UnixStream"
        fi
    fi

    ok "Cargo.toml проверен"
}

# ─────────────────────────────────────────────────────────────────────────────
# ШАГ 7 — tauri.conf.json: прописать externalBin
# ─────────────────────────────────────────────────────────────────────────────
step7_patch_tauri_conf() {
    info "Шаг 7/7 → Обновление tauri.conf.json (externalBin)"

    local sidecar_entry="binaries/sidecar"

    # Проверяем, уже ли прописано
    if grep -q '"binaries/sidecar"' "$TAURI_CONF"; then
        ok "externalBin уже содержит sidecar — пропускаю"
        return
    fi

    # Пробуем через python3 (есть в Termux)
    if command -v python3 &>/dev/null; then
        python3 - "$TAURI_CONF" "$sidecar_entry" << 'PYEOF'
import json, sys

conf_path = sys.argv[1]
entry     = sys.argv[2]

with open(conf_path, 'r') as f:
    conf = json.load(f)

bundle = conf.setdefault('bundle', {})
ext_bin = bundle.setdefault('externalBin', [])

if entry not in ext_bin:
    ext_bin.append(entry)
    print(f"[py] Added '{entry}' to bundle.externalBin")
else:
    print(f"[py] '{entry}' already in externalBin")

with open(conf_path, 'w') as f:
    json.dump(conf, f, indent=2, ensure_ascii=False)
    f.write('\n')
PYEOF
        ok "tauri.conf.json обновлён через python3"

    # Fallback: jq (если установлен)
    elif command -v jq &>/dev/null; then
        local tmp
        tmp=$(mktemp)
        jq --arg e "$sidecar_entry" \
            'if (.bundle.externalBin | index($e)) == null
             then .bundle.externalBin += [$e]
             else . end' "$TAURI_CONF" > "$tmp" && mv "$tmp" "$TAURI_CONF"
        ok "tauri.conf.json обновлён через jq"

    else
        warn "python3 и jq не найдены. Добавь вручную в tauri.conf.json:"
        warn '  "bundle": { "externalBin": ["binaries/sidecar"] }'
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Итоговая проверка
# ─────────────────────────────────────────────────────────────────────────────
final_check() {
    echo ""
    echo -e "${CYAN}── Результат ─────────────────────────────────────────${RESET}"
    [[ -x "$SIDECAR_BIN" ]] && \
        ok "✓ Бинарник:    $SIDECAR_BIN" || \
        die "✗ Бинарник не найден: $SIDECAR_BIN"
    [[ -f "$HTTP_RS" ]] && \
        ok "✓ http.rs:     $HTTP_RS" || \
        warn "? http.rs не найден"
    [[ -f "$LIB_RS" ]] && \
        ok "✓ lib.rs:      $LIB_RS" || \
        warn "? lib.rs не найден"
    grep -q 'externalBin' "$TAURI_CONF" && \
        ok "✓ tauri.conf:  externalBin прописан" || \
        warn "? externalBin не найден в tauri.conf.json"

    echo ""
    echo -e "${GREEN}══════════════════════════════════════════════════════${RESET}"
    echo -e "${GREEN}  Готово! Следующий шаг:                               ${RESET}"
    echo -e "${GREEN}    tauri android build                                 ${RESET}"
    echo -e "${GREEN}══════════════════════════════════════════════════════${RESET}"
    echo ""
    echo -e "  Архитектура:  Rust → UDS → Go sidecar → googleapis"
    echo -e "  TLS preset:   Chrome 131 (JA3/JA4 эмуляция)"
    echo -e "  H2 SETTINGS:  HEADER_TABLE_SIZE=65536, ENABLE_PUSH=0,"
    echo -e "                INITIAL_WINDOW_SIZE=6291456, MAX_HEADER_LIST_SIZE=262144"
    echo -e "  Conn window:  +15663105 (итого 15 MiB)"
    echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Запуск всех шагов
# ─────────────────────────────────────────────────────────────────────────────
step1_install_go
step2_write_go_source
step3_build_go
step4_patch_http_rs
step5_patch_lib_rs
step6_patch_cargo_toml
step7_patch_tauri_conf
final_check
