use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub title: String,
    pub url: String,
    pub visit_count: i64,
}

/// Get Chrome history database path
pub fn get_history_paths() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut paths = Vec::new();

    // Chrome
    let chrome = PathBuf::from(format!(
        "{}/Library/Application Support/Google/Chrome/Default/History",
        home
    ));
    if chrome.exists() {
        paths.push(chrome);
    }

    // Arc
    let arc = PathBuf::from(format!(
        "{}/Library/Application Support/Arc/User Data/Default/History",
        home
    ));
    if arc.exists() {
        paths.push(arc);
    }

    // Brave
    let brave = PathBuf::from(format!(
        "{}/Library/Application Support/BraveSoftware/Brave-Browser/Default/History",
        home
    ));
    if brave.exists() {
        paths.push(brave);
    }

    paths
}

/// Read top visited URLs from a Chrome-compatible History SQLite database
pub fn read_top_history(path: &PathBuf, limit: usize, min_visits: i64) -> Result<Vec<HistoryItem>, String> {
    // Chrome locks the History file while running, so copy it to a temp location first
    let temp_path = std::env::temp_dir().join("flowtab_history_temp");
    std::fs::copy(path, &temp_path)
        .map_err(|e| format!("Cannot read history (browser may be using it): {}", e))?;

    let conn = Connection::open(&temp_path)
        .map_err(|e| format!("Failed to open history: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT url, title, visit_count FROM urls WHERE visit_count >= ?1 ORDER BY visit_count DESC LIMIT ?2",
        )
        .map_err(|e| format!("Failed to query history: {}", e))?;

    let items: Vec<HistoryItem> = stmt
        .query_map(rusqlite::params![min_visits, limit as i64], |row| {
            Ok(HistoryItem {
                url: row.get(0)?,
                title: row.get(1)?,
                visit_count: row.get(2)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .filter(|item| {
            item.url.starts_with("http") && !item.title.is_empty()
        })
        .collect();

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(items)
}
