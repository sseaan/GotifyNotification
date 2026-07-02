use crate::{AppSettings, GotifyMessage, MessageListItem};
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Get the path to the SQLite database file
fn get_db_path(_settings: &AppSettings) -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    path.push("gotify_messages.db");
    path
}

/// Get the data directory for storing settings
fn get_data_dir() -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    path.push("gotify_notification_data");
    std::fs::create_dir_all(&path).ok();
    path
}

/// Determine a reasonable data directory in the user's AppData folder on Windows
fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .ok()
            .map(|p| PathBuf::from(p).join("GotifyNotification"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::data_dir()
    }
}

/// Open the SQLite database, creating tables if needed
fn open_db(settings: &AppSettings) -> Result<Connection, String> {
    let db_path = get_db_path(settings);
    let conn =
        Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS messages (
            id          INTEGER PRIMARY KEY,
            appid       INTEGER NOT NULL,
            message     TEXT NOT NULL,
            title       TEXT NOT NULL,
            priority    INTEGER NOT NULL DEFAULT 0,
            date        TEXT NOT NULL,
            extras      TEXT,
            read        INTEGER NOT NULL DEFAULT 0,
            received_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date DESC);
        CREATE INDEX IF NOT EXISTS idx_messages_read ON messages(read);",
    )
    .map_err(|e| format!("Failed to create tables: {}", e))?;

    Ok(conn)
}

/// Save a message to the database (upsert by id)
pub fn save_message(settings: &AppSettings, msg: &GotifyMessage) -> Result<(), String> {
    let conn = open_db(settings)?;
    let extras = msg
        .extras
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_default();

    conn.execute(
        "INSERT OR REPLACE INTO messages (id, appid, message, title, priority, date, extras, read, received_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            msg.id,
            msg.appid,
            msg.message,
            msg.title,
            msg.priority,
            msg.date,
            extras,
            msg.read as i32,
            msg.received_at,
        ],
    )
    .map_err(|e| format!("Failed to save message: {}", e))?;

    Ok(())
}

/// Get all messages, ordered by date descending
pub fn get_messages(settings: &AppSettings) -> Result<Vec<MessageListItem>, String> {
    let conn = open_db(settings)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, appid, message, title, date, read
             FROM messages
             ORDER BY date DESC
             LIMIT 1000",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(MessageListItem {
                id: row.get(0)?,
                appid: row.get(1)?,
                message: row.get(2)?,
                title: row.get(3)?,
                date: row.get(4)?,
                read: row.get::<_, i32>(5)? != 0,
            })
        })
        .map_err(|e| format!("Failed to query messages: {}", e))?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
    }

    Ok(messages)
}

/// Mark a message as read
pub fn mark_read(settings: &AppSettings, message_id: i64) -> Result<(), String> {
    let conn = open_db(settings)?;
    conn.execute(
        "UPDATE messages SET read = 1 WHERE id = ?1",
        params![message_id],
    )
    .map_err(|e| format!("Failed to mark read: {}", e))?;
    Ok(())
}

/// Delete a single message
pub fn delete_message(settings: &AppSettings, message_id: i64) -> Result<(), String> {
    let conn = open_db(settings)?;
    conn.execute("DELETE FROM messages WHERE id = ?1", params![message_id])
        .map_err(|e| format!("Failed to delete message: {}", e))?;
    Ok(())
}

/// Delete all messages from local storage
pub fn clear_messages(settings: &AppSettings) -> Result<(), String> {
    let conn = open_db(settings)?;
    conn.execute("DELETE FROM messages", [])
        .map_err(|e| format!("Failed to clear messages: {}", e))?;
    Ok(())
}

/// Get count of unread messages
pub fn get_unread_count(settings: &AppSettings) -> Result<i64, String> {
    let conn = open_db(settings)?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE read = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count unread: {}", e))?;
    Ok(count)
}

/// Remove messages older than the configured retention period
pub fn cleanup_old_messages(settings: &AppSettings) {
    if let Ok(conn) = open_db(settings) {
        let days = settings.history_retention_days;
        let _ = conn.execute(
            "DELETE FROM messages WHERE received_at < datetime('now', ?1)",
            params![format!("-{} days", days)],
        );
    }
}

/// Save settings to a JSON file on disk
pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let data_dir = get_data_dir();
    let settings_path = data_dir.join("settings.json");
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&settings_path, json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    Ok(())
}

/// Load settings from the JSON file on disk
pub fn load_settings() -> Option<AppSettings> {
    let data_dir = get_data_dir();
    let settings_path = data_dir.join("settings.json");
    if settings_path.exists() {
        if let Ok(json) = std::fs::read_to_string(&settings_path) {
            serde_json::from_str(&json).ok()
        } else {
            None
        }
    } else {
        None
    }
}
