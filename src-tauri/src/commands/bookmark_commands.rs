use crate::browsers::{bookmarks, history};
use crate::browsers::history::HistoryItem;
use crate::db::queries;
use crate::search::index::SearchIndex;
use crate::state::AppState;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

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
    if let Some(pos) = s.find('#') {
        s = s[..pos].to_string();
    }
    s
}

/// Deduplicate history items by normalized URL, keeping highest visit_count per URL
fn dedup_history_items(items: Vec<HistoryItem>) -> Vec<HistoryItem> {
    let mut map: HashMap<String, HistoryItem> = HashMap::new();
    for item in items {
        let key = normalize_url_for_dedup(&item.url);
        match map.get(&key) {
            Some(existing) if existing.visit_count >= item.visit_count => {}
            _ => { map.insert(key, item); }
        }
    }
    map.into_values().collect()
}

#[tauri::command]
pub fn import_bookmarks(
    state: State<'_, Mutex<AppState>>,
) -> Result<usize, String> {
    let paths = bookmarks::get_bookmark_paths();
    let mut imported = 0usize;

    let app = state.lock().map_err(|e| e.to_string())?;

    for (_browser_type, path) in &paths {
        let items = bookmarks::parse_chrome_bookmarks(
            path.to_str().unwrap_or("")
        )?;

        for item in &items {
            let (pinyin_full, pinyin_initials) = SearchIndex::compute_pinyin(&item.name);
            match queries::insert_page_ignore_dup(
                &app.db,
                &item.name,
                &item.url,
                &pinyin_full,
                &pinyin_initials,
            ) {
                Ok(true) => imported += 1,
                Ok(false) => {}
                Err(_) => {}
            }
        }
    }

    Ok(imported)
}

#[tauri::command]
pub fn analyze_history(
    state: State<'_, Mutex<AppState>>,
) -> Result<usize, String> {
    let history_paths = history::get_history_paths();

    // Collect all items from all browsers
    let mut all_items: Vec<HistoryItem> = Vec::new();
    for path in &history_paths {
        match history::read_top_history(path, 50, 5) {
            Ok(items) => all_items.extend(items),
            Err(_) => {} // skip browsers we can't read
        }
    }

    // Deduplicate by normalized URL
    let unique_items = dedup_history_items(all_items);

    // Insert into database
    let mut imported = 0usize;
    let app = state.lock().map_err(|e| e.to_string())?;

    for item in &unique_items {
        let (pinyin_full, pinyin_initials) = SearchIndex::compute_pinyin(&item.title);
        match queries::insert_page_ignore_dup(
            &app.db,
            &item.title,
            &item.url,
            &pinyin_full,
            &pinyin_initials,
        ) {
            Ok(true) => imported += 1,
            Ok(false) => {}
            Err(_) => {}
        }
    }

    Ok(imported)
}
