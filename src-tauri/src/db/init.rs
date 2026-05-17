use rusqlite::{Connection, Result};
use std::path::Path;

pub fn initialize_database(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS page_index (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            description TEXT DEFAULT '',
            pinyin_full TEXT DEFAULT '',
            pinyin_initials TEXT DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_page_index_name ON page_index(name COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_page_index_url ON page_index(url);
        CREATE INDEX IF NOT EXISTS idx_page_index_pinyin ON page_index(pinyin_initials);

        CREATE TABLE IF NOT EXISTS search_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            page_id INTEGER NOT NULL,
            last_searched_at TEXT NOT NULL DEFAULT (datetime('now')),
            search_count INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY (page_id) REFERENCES page_index(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_search_history_page ON search_history(page_id);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT OR IGNORE INTO settings (key, value) VALUES ('global_hotkey', 'Ctrl+K');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('active_browser', 'chrome');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('window_opacity', '0.95');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'system');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('db_version', '1');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_analyze', 'false');",
    )?;

    Ok(conn)
}
