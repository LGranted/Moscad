use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// ── Глобальное соединение ─────────────────────────────────────────────────────
static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = Connection::open(db_path())
        .expect("Failed to open SQLite database");
    init_schema(&conn).expect("Failed to initialize schema");
    Mutex::new(conn)
});

fn db_path() -> PathBuf {
    PathBuf::from("/data/data/com.lbjlaq.antigravity_tools/files/moscad.db")
}

// ── Схема ─────────────────────────────────────────────────────────────────────
fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;
        PRAGMA busy_timeout=5000;
        PRAGMA synchronous=NORMAL;

        CREATE TABLE IF NOT EXISTS accounts (
            id              TEXT PRIMARY KEY,
            email           TEXT NOT NULL UNIQUE,
            refresh_token   TEXT NOT NULL,
            label           TEXT,
            machine_id      TEXT NOT NULL DEFAULT '',
            is_current      INTEGER NOT NULL DEFAULT 0,
            disabled        INTEGER NOT NULL DEFAULT 0,
            disabled_reason TEXT,
            created_at      INTEGER NOT NULL,
            last_used       INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS proxy_logs (
            id            TEXT PRIMARY KEY,
            account_email TEXT,
            model         TEXT,
            mapped_model  TEXT,
            method        TEXT,
            url           TEXT,
            status        INTEGER,
            duration      INTEGER,
            input_tokens  INTEGER DEFAULT 0,
            output_tokens INTEGER DEFAULT 0,
            protocol      TEXT,
            client_ip     TEXT,
            error         TEXT,
            created_at    INTEGER NOT NULL,
            FOREIGN KEY (account_email) REFERENCES accounts(email) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_proxy_logs_created
            ON proxy_logs (created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_proxy_logs_status
            ON proxy_logs (status);

        CREATE TABLE IF NOT EXISTS ip_access_logs (
            id           TEXT PRIMARY KEY,
            client_ip    TEXT NOT NULL,
            timestamp    INTEGER NOT NULL,
            method       TEXT,
            path         TEXT,
            user_agent   TEXT,
            status       INTEGER,
            duration     INTEGER,
            blocked      INTEGER DEFAULT 0,
            block_reason TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_ip_logs_ip
            ON ip_access_logs (client_ip);
        CREATE INDEX IF NOT EXISTS idx_ip_logs_ts
            ON ip_access_logs (timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_ip_logs_blocked
            ON ip_access_logs (blocked);

        CREATE TABLE IF NOT EXISTS ip_blacklist (
            id          TEXT PRIMARY KEY,
            ip_pattern  TEXT NOT NULL UNIQUE,
            reason      TEXT,
            created_at  INTEGER NOT NULL,
            expires_at  INTEGER,
            created_by  TEXT DEFAULT 'manual',
            hit_count   INTEGER DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_blacklist_pattern
            ON ip_blacklist (ip_pattern);

        CREATE TABLE IF NOT EXISTS ip_whitelist (
            id          TEXT PRIMARY KEY,
            ip_pattern  TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at  INTEGER NOT NULL
        );
    ")
}

// ── Структуры ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id:              String,
    pub email:           String,
    pub refresh_token:   String,
    pub label:           Option<String>,
    pub machine_id:      String,
    pub is_current:      bool,
    pub disabled:        bool,
    pub disabled_reason: Option<String>,
    pub created_at:      i64,
    pub last_used:       i64,
}

/// Генерируем уникальный ANDROID_ID для каждого аккаунта
fn generate_machine_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    let seed = uuid::Uuid::new_v4().to_string();
    seed.hash(&mut hasher);
    chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpBlacklistEntry {
    pub id:          String,
    pub ip_pattern:  String,
    pub reason:      Option<String>,
    pub created_at:  i64,
    pub expires_at:  Option<i64>,
    pub created_by:  String,
    pub hit_count:   i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpWhitelistEntry {
    pub id:          String,
    pub ip_pattern:  String,
    pub description: Option<String>,
    pub created_at:  i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpStats {
    pub total_requests:  u64,
    pub unique_ips:      u64,
    pub blocked_count:   u64,
    pub today_requests:  u64,
    pub blacklist_count: u64,
    pub whitelist_count: u64,
}

// ── Accounts CRUD ─────────────────────────────────────────────────────────────

pub fn list_accounts() -> Result<Vec<Account>, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let conn = &*conn;
    let mut stmt = conn.prepare(
        "SELECT id, email, refresh_token, label, machine_id, is_current,
                disabled, disabled_reason, created_at, last_used
         FROM accounts ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;

    stmt.query_map([], |row| Ok(Account {
        id:              row.get(0)?,
        email:           row.get(1)?,
        refresh_token:   row.get(2)?,
        label:           row.get(3)?,
        machine_id:      row.get::<_, String>(4).unwrap_or_else(|_| generate_machine_id()),
        is_current:      row.get::<_, i64>(5)? == 1,
        disabled:        row.get::<_, i64>(6)? == 1,
        disabled_reason: row.get(7)?,
        created_at:      row.get(8)?,
        last_used:       row.get(9)?,
    }))
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
}

pub fn add_account(email: &str, refresh_token: &str) -> Result<Account, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let now = chrono::Utc::now().timestamp();
    let id  = uuid::Uuid::new_v4().to_string();

    let machine_id = generate_machine_id();

    conn.execute(
        "INSERT INTO accounts (id, email, refresh_token, machine_id, created_at, last_used)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(email) DO UPDATE SET
             refresh_token = excluded.refresh_token,
             last_used     = excluded.last_used",
        params![id, email, refresh_token, machine_id, now, now],
    ).map_err(|e| e.to_string())?;

    Ok(Account {
        id, email: email.to_string(),
        refresh_token: refresh_token.to_string(),
        label: None, machine_id,
        is_current: false,
        disabled: false, disabled_reason: None,
        created_at: now, last_used: now,
    })
}

pub fn delete_account(account_id: &str) -> Result<(), String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    conn.execute("DELETE FROM accounts WHERE id = ?1", params![account_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn switch_account(account_id: &str) -> Result<(), String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    conn.execute("UPDATE accounts SET is_current = 0", [])
        .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE accounts SET is_current = 1, last_used = ?1 WHERE id = ?2",
        params![chrono::Utc::now().timestamp(), account_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_current_account() -> Result<Option<Account>, String> {
    Ok(list_accounts()?.into_iter().find(|a| a.is_current))
}

pub fn update_label(account_id: &str, label: &str) -> Result<(), String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    conn.execute(
        "UPDATE accounts SET label = ?1 WHERE id = ?2",
        params![label, account_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ── IP Blacklist ──────────────────────────────────────────────────────────────

pub fn add_to_blacklist(ip: &str, reason: Option<&str>) -> Result<IpBlacklistEntry, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let id  = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO ip_blacklist (id, ip_pattern, reason, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, 'manual')",
        params![id, ip, reason, now],
    ).map_err(|e| e.to_string())?;
    Ok(IpBlacklistEntry {
        id, ip_pattern: ip.to_string(),
        reason: reason.map(|s| s.to_string()),
        created_at: now, expires_at: None,
        created_by: "manual".to_string(), hit_count: 0,
    })
}

pub fn remove_from_blacklist(id: &str) -> Result<(), String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    conn.execute("DELETE FROM ip_blacklist WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_blacklist() -> Result<Vec<IpBlacklistEntry>, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let conn = \&*conn;
    let mut stmt = conn.prepare(
        "SELECT id, ip_pattern, reason, created_at, expires_at, created_by, hit_count
         FROM ip_blacklist ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    stmt.query_map([], |row| Ok(IpBlacklistEntry {
        id: row.get(0)?, ip_pattern: row.get(1)?,
        reason: row.get(2)?, created_at: row.get(3)?,
        expires_at: row.get(4)?, created_by: row.get(5)?,
        hit_count: row.get(6)?,
    }))
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
}

pub fn is_ip_in_blacklist(ip: &str) -> Result<bool, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let now = chrono::Utc::now().timestamp();
    // Чистим просроченные
    let _ = conn.execute(
        "DELETE FROM ip_blacklist WHERE expires_at IS NOT NULL AND expires_at < ?1",
        [now],
    );
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ip_blacklist WHERE ip_pattern = ?1", [ip], |r| r.get(0)
    ).map_err(|e| e.to_string())?;
    if count > 0 {
        let _ = conn.execute(
            "UPDATE ip_blacklist SET hit_count = hit_count + 1 WHERE ip_pattern = ?1", [ip]
        );
        return Ok(true);
    }
    // CIDR матчинг
    let entries = get_blacklist()?;
    for entry in entries {
        if entry.ip_pattern.contains('/') && cidr_match(ip, &entry.ip_pattern) {
            let _ = conn.execute(
                "UPDATE ip_blacklist SET hit_count = hit_count + 1 WHERE id = ?1", [&entry.id]
            );
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn clear_blacklist() -> Result<(), String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    conn.execute("DELETE FROM ip_blacklist", []).map_err(|e| e.to_string())?;
    Ok(())
}

// ── IP Whitelist ──────────────────────────────────────────────────────────────

pub fn add_to_whitelist(ip: &str, description: Option<&str>) -> Result<IpWhitelistEntry, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let id  = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO ip_whitelist (id, ip_pattern, description, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, ip, description, now],
    ).map_err(|e| e.to_string())?;
    Ok(IpWhitelistEntry {
        id, ip_pattern: ip.to_string(),
        description: description.map(|s| s.to_string()), created_at: now,
    })
}

pub fn remove_from_whitelist(id: &str) -> Result<(), String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    conn.execute("DELETE FROM ip_whitelist WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_whitelist() -> Result<Vec<IpWhitelistEntry>, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let conn = \&*conn;
    let mut stmt = conn.prepare(
        "SELECT id, ip_pattern, description, created_at FROM ip_whitelist ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    stmt.query_map([], |row| Ok(IpWhitelistEntry {
        id: row.get(0)?, ip_pattern: row.get(1)?,
        description: row.get(2)?, created_at: row.get(3)?,
    }))
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
}

pub fn is_ip_in_whitelist(ip: &str) -> Result<bool, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ip_whitelist WHERE ip_pattern = ?1", [ip], |r| r.get(0)
    ).map_err(|e| e.to_string())?;
    if count > 0 { return Ok(true); }
    let entries = get_whitelist()?;
    for entry in entries {
        if entry.ip_pattern.contains('/') && cidr_match(ip, &entry.ip_pattern) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn clear_whitelist() -> Result<(), String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    conn.execute("DELETE FROM ip_whitelist", []).map_err(|e| e.to_string())?;
    Ok(())
}

// ── IP Stats ──────────────────────────────────────────────────────────────────

pub fn get_ip_stats() -> Result<IpStats, String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    let today_start = chrono::Utc::now()
        .date_naive().and_hms_opt(0, 0, 0).unwrap()
        .and_utc().timestamp();

    let (total, unique, blocked, today): (u64, u64, u64, u64) = conn.query_row(
        "SELECT COUNT(*), COUNT(DISTINCT client_ip),
                SUM(CASE WHEN blocked=1 THEN 1 ELSE 0 END),
                SUM(CASE WHEN timestamp >= ?1 THEN 1 ELSE 0 END)
         FROM ip_access_logs",
        [today_start], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    ).map_err(|e| e.to_string())?;

    let bl: u64 = conn.query_row(
        "SELECT COUNT(*) FROM ip_blacklist", [], |r| r.get(0)
    ).map_err(|e| e.to_string())?;
    let wl: u64 = conn.query_row(
        "SELECT COUNT(*) FROM ip_whitelist", [], |r| r.get(0)
    ).map_err(|e| e.to_string())?;

    Ok(IpStats {
        total_requests: total, unique_ips: unique,
        blocked_count: blocked, today_requests: today,
        blacklist_count: bl, whitelist_count: wl,
    })
}

pub fn clear_ip_access_logs() -> Result<(), String> {
    let conn = DB.lock().map_err(|e| e.to_string())?;
    let conn = &*conn;
    conn.execute("DELETE FROM ip_access_logs", []).map_err(|e| e.to_string())?;
    Ok(())
}

// ── CIDR helper ───────────────────────────────────────────────────────────────

fn cidr_match(ip: &str, cidr: &str) -> bool {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 { return false; }
    let prefix_len: u8 = match parts[1].parse() { Ok(p) => p, Err(_) => return false };
    let to_u32 = |s: &str| -> Option<u32> {
        let b: Vec<u8> = s.split('.').filter_map(|x| x.parse().ok()).collect();
        if b.len() != 4 { return None; }
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    };
    let ip_u32  = match to_u32(ip)      { Some(v) => v, None => return false };
    let net_u32 = match to_u32(parts[0]){ Some(v) => v, None => return false };
    let mask = if prefix_len == 0 { 0 } else { !0u32 << (32 - prefix_len) };
    (ip_u32 & mask) == (net_u32 & mask)
}
