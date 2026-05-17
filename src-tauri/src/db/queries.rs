use rusqlite::{params, Connection, Result};

use super::models::PageEntry;

pub fn insert_page(conn: &Connection, name: &str, url: &str, description: &str, pinyin_full: &str, pinyin_initials: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO page_index (name, url, description, pinyin_full, pinyin_initials) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, url, description, pinyin_full, pinyin_initials],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_page(conn: &Connection, id: i64, name: &str, url: &str, description: &str, pinyin_full: &str, pinyin_initials: &str) -> Result<()> {
    conn.execute(
        "UPDATE page_index SET name=?1, url=?2, description=?3, pinyin_full=?4, pinyin_initials=?5, updated_at=datetime('now') WHERE id=?6",
        params![name, url, description, pinyin_full, pinyin_initials, id],
    )?;
    Ok(())
}

pub fn delete_page(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM page_index WHERE id=?1", params![id])?;
    Ok(())
}

pub fn get_page_by_id(conn: &Connection, id: i64) -> Result<Option<PageEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, description, pinyin_full, pinyin_initials, created_at, updated_at FROM page_index WHERE id=?1"
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(PageEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            description: row.get(3)?,
            pinyin_full: row.get(4)?,
            pinyin_initials: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    match rows.next() {
        Some(Ok(entry)) => Ok(Some(entry)),
        _ => Ok(None),
    }
}

pub fn get_all_pages(conn: &Connection) -> Result<Vec<PageEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, description, pinyin_full, pinyin_initials, created_at, updated_at FROM page_index ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PageEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            description: row.get(3)?,
            pinyin_full: row.get(4)?,
            pinyin_initials: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key=?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
    match rows.next() {
        Some(Ok(val)) => Ok(Some(val)),
        _ => Ok(None),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn insert_page_ignore_dup(conn: &Connection, name: &str, url: &str, pinyin_full: &str, pinyin_initials: &str) -> Result<bool> {
    let normalized = normalize_url_for_dedup(url);

    // Check for duplicate by comparing normalized URLs
    let all_pages = get_all_pages(conn)?;
    for page in &all_pages {
        if normalize_url_for_dedup(&page.url) == normalized {
            return Ok(false);
        }
    }

    insert_page(conn, name, url, "", pinyin_full, pinyin_initials)?;
    Ok(true)
}

/// Remove duplicate pages, keeping the oldest entry per normalized URL.
/// Returns the number of duplicates removed.
pub fn clean_duplicates(conn: &Connection) -> Result<usize> {
    let pages = get_all_pages(conn)?;
    let mut seen: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut removed = 0usize;

    for page in &pages {
        let norm = normalize_url_for_dedup(&page.url);
        if let Some(&keep_id) = seen.get(&norm) {
            // This is a duplicate - delete it
            conn.execute("DELETE FROM page_index WHERE id=?1", rusqlite::params![page.id])?;
            removed += 1;
        } else {
            seen.insert(norm, page.id);
        }
    }

    Ok(removed)
}

/// Normalize URL for deduplication: lowercase, remove trailing /, standardize www and protocol
fn normalize_url_for_dedup(url: &str) -> String {
    let mut s = url.trim().to_lowercase();
    // Remove protocol prefix for comparison
    if s.starts_with("https://") {
        s = s[8..].to_string();
    } else if s.starts_with("http://") {
        s = s[7..].to_string();
    }
    // Remove www.
    if s.starts_with("www.") {
        s = s[4..].to_string();
    }
    // Remove trailing slash
    s = s.trim_end_matches('/').to_string();
    // Remove trailing hash
    if let Some(pos) = s.find('#') {
        s = s[..pos].to_string();
    }
    s
}
