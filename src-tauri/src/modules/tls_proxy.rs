// src-tauri/src/modules/tls_proxy.rs
//
// TLS Traffic Normalization Proxy
// ────────────────────────────────────────────────────────────────────────────
// Архитектура:
//
//   reqwest::Client  ──TCP──►  LocalProxy (127.0.0.1:PORT)
//                                  │
//                                  ▼
//                          ClientHello parser
//                          Extension reorder  (→ Chromium order)
//                          GREASE injector
//                                  │
//                              TCP forward
//                                  │
//                                  ▼
//                         Corporate Server (real target)
//
// Зависимости (Cargo.toml):
//
//   [target.'cfg(target_os = "android")'.dependencies]
//   tokio          = { version = "1", features = ["net", "io-util", "rt-multi-thread", "sync", "macros"] }
//   reqwest        = { version = "0.12", default-features = false, features = ["rustls-tls-manual-roots"] }
//   tracing        = "0.1"
//   thiserror      = "1"
//   once_cell      = "1"
//
// Намеренно НЕ используем tls-parser: он тянет nom и не нужен —
// ClientHello парсится вручную за ~100 строк и даёт полный контроль над байтами.
// ────────────────────────────────────────────────────────────────────────────

use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};

use once_cell::sync::OnceCell;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Notify,
};
use tracing::{debug, error, info, warn};

// ═══════════════════════════════════════════════════════════════════════════
//  ОШИБКИ
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Proxy not initialized")]
    NotInitialized,
    #[error("Malformed ClientHello: {0}")]
    ParseError(String),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

// ═══════════════════════════════════════════════════════════════════════════
//  ГЛОБАЛЬНЫЙ SINGLETON ПРОКСИ
// ═══════════════════════════════════════════════════════════════════════════

static PROXY: OnceCell<Arc<TlsNormProxy>> = OnceCell::new();

/// Инициализирует и запускает прокси. Вызывать один раз при старте приложения.
pub async fn init_proxy(target_host: String, target_port: u16) -> Result<u16, ProxyError> {
    let proxy = Arc::new(TlsNormProxy::new(target_host, target_port).await?);
    let port = proxy.local_port();

    let proxy_clone = Arc::clone(&proxy);
    tokio::spawn(async move {
        proxy_clone.run().await;
    });

    PROXY
        .set(proxy)
        .map_err(|_| ProxyError::Io(std::io::Error::other("Proxy already initialized")))?;

    info!("TLS normalization proxy started on 127.0.0.1:{port}");
    Ok(port)
}

/// Возвращает `reqwest::Client`, проксированный через локальный TLS-нормализатор.
pub async fn get_proxied_client(
    target_host: &str,
    target_port: u16,
) -> Result<reqwest::Client, ProxyError> {
    // Запускаем если ещё не запущен
    let port = if let Some(proxy) = PROXY.get() {
        proxy.local_port()
    } else {
        init_proxy(target_host.to_string(), target_port).await?
    };

    // reqwest → наш локальный прокси (plain TCP, без TLS на этом участке)
    // TLS-рукопожатие происходит между прокси и сервером с нашим патченым ClientHello
    let proxy_url = format!("http://127.0.0.1:{port}");

    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::https(&proxy_url)?)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    Ok(client)
}

// ═══════════════════════════════════════════════════════════════════════════
//  СТРУКТУРА ПРОКСИ
// ═══════════════════════════════════════════════════════════════════════════

pub struct TlsNormProxy {
    listener: TcpListener,
    target_host: String,
    target_port: u16,
    shutdown: Arc<Notify>,
}

impl TlsNormProxy {
    async fn new(target_host: String, target_port: u16) -> Result<Self, ProxyError> {
        // Биндимся на случайный порт — ОС выберет свободный
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        Ok(Self {
            listener,
            target_host,
            target_port,
            shutdown: Arc::new(Notify::new()),
        })
    }

    pub fn local_port(&self) -> u16 {
        self.listener
            .local_addr()
            .map(|a| a.port())
            .unwrap_or(0)
    }

    pub fn shutdown(&self) {
        self.shutdown.notify_one();
    }

    /// Основной accept-loop
    async fn run(&self) {
        loop {
            tokio::select! {
                _ = self.shutdown.notified() => {
                    info!("TLS proxy shutting down");
                    break;
                }
                result = self.listener.accept() => {
                    match result {
                        Ok((stream, peer)) => {
                            debug!("New connection from {peer}");
                            let host = self.target_host.clone();
                            let port = self.target_port;
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, host, port).await {
                                    warn!("Connection error: {e}");
                                }
                            });
                        }
                        Err(e) => error!("Accept error: {e}"),
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ОБРАБОТКА ОДНОГО TCP-СОЕДИНЕНИЯ
// ═══════════════════════════════════════════════════════════════════════════

async fn handle_connection(
    mut local: TcpStream,
    target_host: String,
    target_port: u16,
) -> Result<(), ProxyError> {
    // 1. Читаем первый TLS-запись — должна быть ClientHello
    let mut buf = vec![0u8; 16 * 1024]; // 16 KiB — достаточно для любого ClientHello
    let n = local.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    buf.truncate(n);

    // 2. Патчим ClientHello если это TLS-handshake
    let patched = if looks_like_client_hello(&buf) {
        match patch_client_hello(buf.clone()) {
            Ok(p) => {
                debug!("ClientHello patched: {} → {} bytes", n, p.len());
                p
            }
            Err(e) => {
                warn!("Failed to patch ClientHello ({e}), forwarding as-is");
                buf
            }
        }
    } else {
        buf
    };

    // 3. Соединяемся с реальным сервером
    let addr = format!("{target_host}:{target_port}");
    let mut remote = TcpStream::connect(&addr).await?;
    debug!("Connected to remote {addr}");

    // 4. Отправляем патченый первый пакет
    remote.write_all(&patched).await?;

    // 5. Прозрачная пересылка всего остального трафика в обе стороны
    let (mut lr, mut lw) = local.into_split();
    let (mut rr, mut rw) = remote.into_split();

    let client_to_server = tokio::io::copy(&mut lr, &mut rw);
    let server_to_client = tokio::io::copy(&mut rr, &mut lw);

    // Ждём закрытия любой из сторон
    tokio::select! {
        res = client_to_server => {
            debug!("Client→Server done: {:?}", res);
        }
        res = server_to_client => {
            debug!("Server→Client done: {:?}", res);
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
//  TLS RECORD PARSER  (без внешних зависимостей)
// ═══════════════════════════════════════════════════════════════════════════
//
// Структура TLS ClientHello (RFC 8446 §4.1.2):
//
//  TLS Record Layer:
//    [0]    ContentType  = 0x16 (handshake)
//    [1..2] Version      = 0x0301 (TLS 1.0 для совместимости)
//    [3..4] Length       (u16 BE)
//
//  Handshake Header:
//    [5]    HandshakeType = 0x01 (client_hello)
//    [6..8] Length         (u24 BE)
//
//  ClientHello body:
//    [9..10]  ProtocolVersion  (u16)
//    [11..42] Random           (32 bytes)
//    [43]     SessionID length (u8)
//    [44..44+SID_LEN] SessionID
//    ... CipherSuites length (u16), CipherSuites
//    ... CompressionMethods length (u8), methods
//    ... Extensions length (u16), Extensions[]
//
//  Extension:
//    [0..1] Type   (u16 BE)
//    [2..3] Length (u16 BE)
//    [4..4+len] Data

fn looks_like_client_hello(buf: &[u8]) -> bool {
    buf.len() > 9
        && buf[0] == 0x16  // TLS Handshake
        && buf[5] == 0x01  // ClientHello
}

/// Возвращает изменённый ClientHello с Chromium-совместимым порядком расширений
fn patch_client_hello(mut buf: Vec<u8>) -> Result<Vec<u8>, ProxyError> {
    // ── Находим начало блока Extensions ──────────────────────────────────

    let ext_block_offset = find_extensions_offset(&buf)?;

    // Extensions length — 2 байта в big-endian
    if ext_block_offset + 2 > buf.len() {
        return Err(ProxyError::ParseError("Buffer too short for ext length".into()));
    }
    let ext_len = u16::from_be_bytes([buf[ext_block_offset], buf[ext_block_offset + 1]]) as usize;
    let ext_data_start = ext_block_offset + 2;
    let ext_data_end = ext_data_start + ext_len;

    if ext_data_end > buf.len() {
        return Err(ProxyError::ParseError(
            format!("Extension block out of bounds: {ext_data_end} > {}", buf.len())
        ));
    }

    // ── Парсим расширения в Vec<(type, data)> ────────────────────────────

    let raw_exts = &buf[ext_data_start..ext_data_end];
    let mut extensions = parse_extensions(raw_exts)?;

    // ── Добавляем GREASE-расширение если его нет ─────────────────────────
    inject_grease_if_missing(&mut extensions);

    // ── Переупорядочиваем по Chromium-профилю ────────────────────────────
    reorder_extensions_chromium(&mut extensions);

    // ── Сериализуем обратно ───────────────────────────────────────────────
    let new_ext_bytes = serialize_extensions(&extensions);
    let new_ext_len = new_ext_bytes.len();

    // Собираем новый буфер
    let mut result = buf[..ext_data_start].to_vec();
    result.extend_from_slice(&new_ext_bytes);
    // Остаток после блока расширений (обычно пустой, но для корректности)
    result.extend_from_slice(&buf[ext_data_end..]);

    // Патчим длины в заголовках (они могли измениться из-за GREASE)
    fix_lengths(&mut result, ext_block_offset, new_ext_len)?;

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────
//  Вспомогательные функции парсинга
// ─────────────────────────────────────────────────────────────────────────

/// Находит байтовый offset поля "Extensions Length" внутри ClientHello
fn find_extensions_offset(buf: &[u8]) -> Result<usize, ProxyError> {
    let mut pos = 9usize; // после TLS record (5) + handshake header (4)

    // ProtocolVersion (2)
    pos += 2;
    // Random (32)
    pos += 32;

    check_bounds(buf, pos, 1, "SessionID length")?;
    let sid_len = buf[pos] as usize;
    pos += 1 + sid_len;

    check_bounds(buf, pos, 2, "CipherSuites length")?;
    let cs_len = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
    pos += 2 + cs_len;

    check_bounds(buf, pos, 1, "CompressionMethods length")?;
    let cm_len = buf[pos] as usize;
    pos += 1 + cm_len;

    // Здесь должно быть поле Extensions Length
    check_bounds(buf, pos, 2, "Extensions block")?;
    Ok(pos)
}

fn check_bounds(buf: &[u8], pos: usize, needed: usize, field: &str) -> Result<(), ProxyError> {
    if pos + needed > buf.len() {
        Err(ProxyError::ParseError(format!(
            "Buffer too short at '{field}': pos={pos}, need={needed}, len={}",
            buf.len()
        )))
    } else {
        Ok(())
    }
}

/// Парсит блок расширений в вектор (type_u16, data_bytes)
fn parse_extensions(raw: &[u8]) -> Result<Vec<(u16, Vec<u8>)>, ProxyError> {
    let mut exts = Vec::new();
    let mut pos = 0;

    while pos + 4 <= raw.len() {
        let ext_type = u16::from_be_bytes([raw[pos], raw[pos + 1]]);
        let ext_len = u16::from_be_bytes([raw[pos + 2], raw[pos + 3]]) as usize;
        pos += 4;

        if pos + ext_len > raw.len() {
            return Err(ProxyError::ParseError(format!(
                "Extension {ext_type:#06x} data out of bounds"
            )));
        }

        exts.push((ext_type, raw[pos..pos + ext_len].to_vec()));
        pos += ext_len;
    }

    Ok(exts)
}

fn serialize_extensions(exts: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (ext_type, data) in exts {
        out.extend_from_slice(&ext_type.to_be_bytes());
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(data);
    }
    out
}

// ═══════════════════════════════════════════════════════════════════════════
//  CHROMIUM EXTENSION ORDER
// ═══════════════════════════════════════════════════════════════════════════
//
// Источник: https://tls.browserleaks.com / Chromium 120 ClientHello дамп
//
// Порядок расширений в Chromium 120 (TLS 1.3):
//   GREASE (0xXAXA)              — случайное GREASE-значение
//   0x0000  server_name (SNI)
//   0x0017  extended_master_secret
//   0xff01  renegotiation_info
//   0x000a  supported_groups
//   0x000b  ec_point_formats
//   0x0023  session_ticket
//   0x0010  application_layer_protocol_negotiation (ALPN)
//   0x0005  status_request
//   0x0012  signed_certificate_timestamp
//   0x000d  signature_algorithms
//   0x0012  signed_certificate_timestamp (dedup ниже)
//   0x0033  key_share
//   0x002b  supported_versions
//   0x002d  psk_key_exchange_modes
//   0x001b  compress_certificate
//   GREASE (encrypted_client_hello placeholder)
//   0x0015  padding
//
// Расширения, которых нет в оригинальном ClientHello, пропускаются.
// Расширения вне этого списка добавляются В КОНЕЦ (не теряем данные).

const CHROMIUM_EXT_ORDER: &[u16] = &[
    0xFF00, // GREASE placeholder (будет заменён реальным GREASE-значением при inject)
    0x0000, // server_name
    0x0017, // extended_master_secret
    0xFF01, // renegotiation_info
    0x000A, // supported_groups
    0x000B, // ec_point_formats
    0x0023, // session_ticket
    0x0010, // alpn
    0x0005, // status_request
    0x0012, // signed_certificate_timestamp
    0x000D, // signature_algorithms
    0x0033, // key_share
    0x002B, // supported_versions
    0x002D, // psk_key_exchange_modes
    0x001B, // compress_certificate
    0x0015, // padding
];

fn reorder_extensions_chromium(exts: &mut Vec<(u16, Vec<u8>)>) {
    let original: Vec<(u16, Vec<u8>)> = std::mem::take(exts);

    // Индекс для быстрого поиска по типу
    // (type → last occurrence, т.к. GREASE может встречаться несколько раз)
    let mut map: std::collections::HashMap<u16, Vec<u8>> = original
        .clone()
        .into_iter()
        .collect();

    // GREASE-расширения: тип с паттерном 0x?A?A
    let grease_exts: Vec<(u16, Vec<u8>)> = original
        .iter()
        .filter(|(t, _)| is_grease(*t))
        .cloned()
        .collect();

    let mut result: Vec<(u16, Vec<u8>)> = Vec::with_capacity(original.len());
    let mut placed: std::collections::HashSet<u16> = std::collections::HashSet::new();

    for &target_type in CHROMIUM_EXT_ORDER {
        if target_type == 0xFF00 {
            // Слот для GREASE — вставляем все GREASE-расширения
            for g in &grease_exts {
                if !placed.contains(&g.0) {
                    result.push(g.clone());
                    placed.insert(g.0);
                }
            }
        } else if let Some(data) = map.remove(&target_type) {
            result.push((target_type, data));
            placed.insert(target_type);
        }
    }

    // Всё что не вошло в Chromium-список — добавляем в конец
    for (t, d) in original {
        if !placed.contains(&t) {
            result.push((t, d));
            placed.insert(t);
        }
    }

    *exts = result;
}

// ═══════════════════════════════════════════════════════════════════════════
//  GREASE  (RFC 8701)
// ═══════════════════════════════════════════════════════════════════════════
//
// GREASE-значения для расширений: 0x0A0A, 0x1A1A, ..., 0xFAFA
// Chromium добавляет одно случайное GREASE-расширение с пустыми данными.

static GREASE_COUNTER: AtomicU16 = AtomicU16::new(0);

fn next_grease_ext_type() -> u16 {
    // Циклически выбираем из 16 допустимых GREASE-значений
    let idx = GREASE_COUNTER.fetch_add(1, Ordering::Relaxed) % 16;
    0x0A0A + (idx * 0x1010)
}

fn is_grease(t: u16) -> bool {
    // GREASE: оба байта одинаковы и кратны 0x0A с паттерном 0x?A?A
    let hi = (t >> 8) as u8;
    let lo = (t & 0xFF) as u8;
    hi == lo && (hi & 0x0F) == 0x0A
}

fn inject_grease_if_missing(exts: &mut Vec<(u16, Vec<u8>)>) {
    let has_grease = exts.iter().any(|(t, _)| is_grease(*t));
    if !has_grease {
        let grease_type = next_grease_ext_type();
        // GREASE-расширение с пустыми данными — Chromium-поведение
        exts.insert(0, (grease_type, vec![]));
        debug!("Injected GREASE extension {grease_type:#06x}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ФИКС ДЛИН В TLS-ЗАПИСИ
// ═══════════════════════════════════════════════════════════════════════════

fn fix_lengths(buf: &mut Vec<u8>, ext_block_offset: usize, new_ext_len: usize) -> Result<(), ProxyError> {
    // Extensions Length (2 байта) по ext_block_offset
    let ext_len_bytes = (new_ext_len as u16).to_be_bytes();
    buf[ext_block_offset] = ext_len_bytes[0];
    buf[ext_block_offset + 1] = ext_len_bytes[1];

    // Handshake Length (u24 BE) по offset 6
    // = всё от байта 9 до конца buf
    let hs_body_len = buf.len() - 9;
    buf[6] = ((hs_body_len >> 16) & 0xFF) as u8;
    buf[7] = ((hs_body_len >> 8) & 0xFF) as u8;
    buf[8] = (hs_body_len & 0xFF) as u8;

    // TLS Record Length (u16 BE) по offset 3
    // = всё от байта 5 до конца buf
    let record_len = buf.len() - 5;
    if record_len > 0xFFFF {
        return Err(ProxyError::ParseError("Record too large".into()));
    }
    let rec_len_bytes = (record_len as u16).to_be_bytes();
    buf[3] = rec_len_bytes[0];
    buf[4] = rec_len_bytes[1];

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
//  ТЕСТЫ
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// Минимальный синтетический ClientHello для unit-тестов
    fn make_fake_client_hello(ext_types: &[(u16, Vec<u8>)]) -> Vec<u8> {
        let mut body = Vec::new();
        // ProtocolVersion
        body.extend_from_slice(&[0x03, 0x03]);
        // Random (32 bytes)
        body.extend_from_slice(&[0xAB; 32]);
        // SessionID length = 0
        body.push(0x00);
        // CipherSuites: 1 suite = TLS_AES_128_GCM_SHA256
        body.extend_from_slice(&[0x00, 0x02, 0x13, 0x01]);
        // CompressionMethods: 1 = null
        body.extend_from_slice(&[0x01, 0x00]);

        // Extensions
        let mut ext_bytes = Vec::new();
        for (t, d) in ext_types {
            ext_bytes.extend_from_slice(&t.to_be_bytes());
            ext_bytes.extend_from_slice(&(d.len() as u16).to_be_bytes());
            ext_bytes.extend_from_slice(d);
        }
        body.extend_from_slice(&(ext_bytes.len() as u16).to_be_bytes());
        body.extend_from_slice(&ext_bytes);

        // Handshake header
        let hs_len = body.len();
        let mut hs = vec![0x01]; // ClientHello
        hs.push(((hs_len >> 16) & 0xFF) as u8);
        hs.push(((hs_len >> 8) & 0xFF) as u8);
        hs.push((hs_len & 0xFF) as u8);
        hs.extend_from_slice(&body);

        // TLS record
        let rec_len = hs.len();
        let mut record = vec![0x16, 0x03, 0x01];
        record.push(((rec_len >> 8) & 0xFF) as u8);
        record.push((rec_len & 0xFF) as u8);
        record.extend_from_slice(&hs);
        record
    }

    #[test]
    fn test_looks_like_client_hello() {
        let ch = make_fake_client_hello(&[]);
        assert!(looks_like_client_hello(&ch));
        assert!(!looks_like_client_hello(&[0x17, 0x03, 0x03, 0x00, 0x05]));
    }

    #[test]
    fn test_grease_detection() {
        assert!(is_grease(0x0A0A));
        assert!(is_grease(0xFAFA));
        assert!(is_grease(0x2A2A));
        assert!(!is_grease(0x0000));
        assert!(!is_grease(0x0010));
    }

    #[test]
    fn test_patch_roundtrip_lengths() {
        let exts = vec![
            (0x0010u16, vec![0x00, 0x02, 0x68, 0x32]), // ALPN: h2
            (0x0000u16, vec![0x00, 0x00, 0x0A, 0x00, 0x08, 0x00, 0x06, // SNI (fake)
                              0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65]),
            (0x002Bu16, vec![0x03, 0x04, 0x03, 0x03]),  // supported_versions
        ];
        let original = make_fake_client_hello(&exts);
        let patched = patch_client_hello(original.clone()).unwrap();

        // TLS record layer lengths должны быть консистентны
        let rec_len = u16::from_be_bytes([patched[3], patched[4]]) as usize;
        assert_eq!(rec_len, patched.len() - 5, "TLS Record length mismatch");

        let hs_len = ((patched[6] as usize) << 16)
            | ((patched[7] as usize) << 8)
            | (patched[8] as usize);
        assert_eq!(hs_len, patched.len() - 9, "Handshake length mismatch");
    }

    #[test]
    fn test_chromium_order_sni_first_after_grease() {
        let exts = vec![
            (0x000Du16, vec![0x00, 0x02, 0x04, 0x01]), // signature_algorithms (должен уйти вниз)
            (0x0000u16, vec![0x00; 8]),                 // SNI (должен быть 2-м после GREASE)
            (0x0010u16, vec![0x00, 0x02, 0x68, 0x32]), // ALPN
        ];
        let original = make_fake_client_hello(&exts);
        let patched = patch_client_hello(original).unwrap();

        let ext_offset = find_extensions_offset(&patched).unwrap();
        let ext_len = u16::from_be_bytes([patched[ext_offset], patched[ext_offset + 1]]) as usize;
        let parsed = parse_extensions(&patched[ext_offset + 2..ext_offset + 2 + ext_len]).unwrap();

        // Первое расширение — GREASE
        assert!(is_grease(parsed[0].0), "First ext must be GREASE, got {:#06x}", parsed[0].0);
        // Второе — SNI
        assert_eq!(parsed[1].0, 0x0000, "Second ext must be SNI");
    }
}
