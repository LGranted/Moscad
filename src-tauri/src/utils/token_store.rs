//! Memory Token Injection
//! Глобальный store токенов — бесшовное обновление без перезапуска процесса

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct TokenEntry {
    pub account_id:    String,
    pub email:         String,
    pub access_token:  String,
    pub refresh_token: String,
    pub expires_at:    i64, // Unix timestamp
}

impl TokenEntry {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        // Обновляем за 60 секунд до истечения
        now >= self.expires_at - 60
    }
}

// ── Максимум аккаунтов в памяти одновременно ─────────────────────────────────
const LRU_CAPACITY: usize = 10;

// ── LRU очередь (от старых к новым) ──────────────────────────────────────────
static LRU_QUEUE: Lazy<Mutex<std::collections::VecDeque<String>>> =
    Lazy::new(|| Mutex::new(std::collections::VecDeque::new()));

// ── Глобальный store ──────────────────────────────────────────────────────────
static TOKEN_STORE: Lazy<Mutex<HashMap<String, TokenEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// ── Текущий активный аккаунт ──────────────────────────────────────────────────
static CURRENT_ACCOUNT_ID: Lazy<Mutex<Option<String>>> =
    Lazy::new(|| Mutex::new(None));

// ── API ───────────────────────────────────────────────────────────────────────

/// Загрузить токен в память (вызывается после OAuth или switch_account)
/// LRU: если в памяти уже LRU_CAPACITY аккаунтов — вытесняем самый старый
pub fn store_token(entry: TokenEntry) {
    let id = entry.account_id.clone();

    // Обновляем LRU очередь
    if let Ok(mut queue) = LRU_QUEUE.lock() {
        // Убираем если уже есть (обновляем позицию)
        queue.retain(|x| x != &id);
        // Добавляем в конец (самый свежий)
        queue.push_back(id.clone());

        // Вытесняем самый старый если превысили лимит
        if queue.len() > LRU_CAPACITY {
            if let Some(evicted_id) = queue.pop_front() {
                if let Ok(mut store) = TOKEN_STORE.lock() {
                    store.remove(&evicted_id);
                }
                log::info!("[LRU] Evicted token for account: {}", evicted_id);
            }
        }
    }

    if let Ok(mut store) = TOKEN_STORE.lock() {
        store.insert(id, entry);
    }
}

/// Текущий размер пула в памяти
pub fn pool_size() -> usize {
    TOKEN_STORE.lock().map(|s| s.len()).unwrap_or(0)
}

/// Установить текущий активный аккаунт
pub fn set_current(account_id: &str) {
    if let Ok(mut cur) = CURRENT_ACCOUNT_ID.lock() {
        *cur = Some(account_id.to_string());
    }
}

/// Удалить токен из памяти (при delete_account)
pub fn remove_token(account_id: &str) {
    if let Ok(mut store) = TOKEN_STORE.lock() {
        store.remove(account_id);
    }
    if let Ok(mut cur) = CURRENT_ACCOUNT_ID.lock() {
        if cur.as_deref() == Some(account_id) {
            *cur = None;
        }
    }
}

/// Получить свежий access token для текущего аккаунта.
/// Если токен истёк — автоматически обновляет через refresh token.
pub async fn get_fresh_token() -> Option<String> {
    let account_id = CURRENT_ACCOUNT_ID.lock().ok()?.clone()?;

    // Берём копию entry чтобы не держать lock во время async вызова
    let entry = {
        let store = TOKEN_STORE.lock().ok()?;
        store.get(&account_id)?.clone()
    };

    if !entry.is_expired() {
        return Some(entry.access_token.clone());
    }

    // Токен истёк — обновляем
    log::info!("[TokenStore] Access token expired for {}, refreshing...", entry.email);

    match crate::modules::oauth::refresh_access_token(&entry.refresh_token, None).await {
        Ok(resp) => {
            let new_expires = chrono::Utc::now().timestamp() + resp.expires_in;
            let new_entry = TokenEntry {
                account_id:    entry.account_id.clone(),
                email:         entry.email.clone(),
                access_token:  resp.access_token.clone(),
                refresh_token: resp.refresh_token
                    .unwrap_or_else(|| entry.refresh_token.clone()),
                expires_at:    new_expires,
            };
            if let Ok(mut store) = TOKEN_STORE.lock() {
                store.insert(entry.account_id.clone(), new_entry);
            }
            log::info!("[TokenStore] Token refreshed for {}", entry.email);
            Some(resp.access_token)
        }
        Err(e) => {
            log::error!("[TokenStore] Token refresh failed for {}: {}", entry.email, e);
            // Возвращаем старый токен — пусть сервер вернёт 401
            Some(entry.access_token.clone())
        }
    }
}
