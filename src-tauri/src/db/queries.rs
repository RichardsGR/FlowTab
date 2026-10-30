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

/// Remove duplicate pages, keeping the shortest URL per normalized URL.
/// Returns the number of duplicates removed.
pub fn clean_duplicates(conn: &Connection) -> Result<usize> {
    let pages = get_all_pages(conn)?;
    let mut keep: std::collections::HashMap<String, &PageEntry> =
        std::collections::HashMap::new();

    for page in &pages {
        let norm = normalize_url_for_dedup(&page.url);
        let should_replace = match keep.get(&norm) {
            Some(current) => current.url.len() > page.url.len(),
            None => true,
        };
        if should_replace {
            keep.insert(norm, page);
        }
    }

    let keep_ids: std::collections::HashSet<i64> = keep.values().map(|p| p.id).collect();
    let mut removed = 0usize;
    for page in &pages {
        if !keep_ids.contains(&page.id) {
            conn.execute("DELETE FROM page_index WHERE id=?1", params![page.id])?;
            removed += 1;
        }
    }

    Ok(removed)
}

/// Find an existing page whose URL matches after normalization.
pub fn find_page_by_normalized_url(
    conn: &Connection,
    url: &str,
) -> Result<Option<PageEntry>> {
    let key = normalize_url_for_dedup(url);
    for page in get_all_pages(conn)? {
        if normalize_url_for_dedup(&page.url) == key {
            return Ok(Some(page));
        }
    }
    Ok(None)
}

/// Normalize URL for deduplication: lowercase, strip protocol, www, query,
/// fragment, and trailing slash.
fn normalize_url_for_dedup(url: &str) -> String {
    let mut s = url.trim().to_lowercase();

    if let Some(stripped) = s.strip_prefix("https://") {
        s = stripped.to_string();
    } else if let Some(stripped) = s.strip_prefix("http://") {
        s = stripped.to_string();
    }

    if let Some(stripped) = s.strip_prefix("www.") {
        s = stripped.to_string();
    }

    s = s.trim_end_matches('/').to_string();
    if let Some(pos) = s.find(|c| c == '?' || c == '#') {
        s = s[..pos].to_string();
    }
    s.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE page_index (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                description TEXT DEFAULT '',
                pinyin_full TEXT DEFAULT '',
                pinyin_initials TEXT DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn normalizes_query_fragment_protocol_and_www() {
        assert_eq!(
            normalize_url_for_dedup("https://www.Example.com/path/?tab=1#section"),
            "example.com/path"
        );
        assert_eq!(
            normalize_url_for_dedup("http://example.com/path"),
            "example.com/path"
        );
    }

    #[test]
    fn clean_duplicates_keeps_shortest_url() {
        let conn = test_conn();
        insert_page(&conn, "Base", "https://example.com/path", "", "", "").unwrap();
        insert_page(&conn, "Variant", "https://example.com/path?tab=1", "", "", "").unwrap();
        insert_page(&conn, "Other", "https://other.com/", "", "", "").unwrap();

        let removed = clean_duplicates(&conn).unwrap();
        assert_eq!(removed, 1);

        let pages = get_all_pages(&conn).unwrap();
        assert_eq!(pages.len(), 2);
        assert!(pages.iter().any(|p| p.url == "https://example.com/path"));
        assert!(!pages.iter().any(|p| p.url.contains("tab=1")));
    }

    #[test]
    fn finds_existing_page_by_normalized_url() {
        let conn = test_conn();
        insert_page(&conn, "Base", "https://example.com/path", "", "", "").unwrap();

        let found =
            find_page_by_normalized_url(&conn, "https://example.com/path?tab=1").unwrap();
        assert_eq!(found.map(|p| p.name), Some("Base".to_string()));
    }
}
