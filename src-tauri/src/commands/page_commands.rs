use crate::browsers::{self, BrowserType, OpenTab};
use crate::db::models::PageEntry;
use crate::db::queries;
use crate::search::index::SearchIndex;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entry: PageEntry,
    pub score: u16,
    pub is_open: bool,
    /// "indexed" = from database, "open_tab" = not in index but currently open
    pub source: String,
}

#[tauri::command]
pub fn search_pages(
    query: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<SearchResult>, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    let entries = queries::get_all_pages(&app.db).map_err(|e| e.to_string())?;
    let browser_key = queries::get_setting(&app.db, "active_browser")
        .ok()
        .flatten()
        .unwrap_or_else(|| "chrome".into());
    drop(app);

    // Try to get open tabs (non-fatal if it fails)
    let open_tabs: Vec<OpenTab> = get_open_tabs_silent(&browser_key).unwrap_or_default();

    let mut index = SearchIndex::new();
    let scored = index.search(&query, &entries);

    // Mark indexed entries as open if their URL matches an open tab
    let mut results: Vec<SearchResult> = scored
        .into_iter()
        .map(|s| {
            let is_open = is_url_open(&s.entry.url, &open_tabs);
            SearchResult {
                entry: s.entry,
                score: s.score,
                is_open,
                source: "indexed".into(),
            }
        })
        .collect();

    // Add open tabs that match the query but aren't in the index
    let indexed_urls: Vec<String> = results.iter().map(|r| normalize_url(&r.entry.url)).collect();

    for tab in &open_tabs {
        let tab_url = normalize_url(&tab.url);
        if indexed_urls.contains(&tab_url) {
            continue; // Already in results
        }

        // Check if the tab title/URL matches the query
        let tab_title = &tab.title;
        let mut max_score: u16 = 0;

        if tab_title.to_lowercase().contains(&query.to_lowercase()) {
            max_score = 800;
        } else if let Some(s) = index.search_one(&query, tab_title) {
            max_score = s;
        } else if tab_url.contains(&query.to_lowercase()) {
            max_score = 600;
        }

        if max_score > 0 {
            let fake_entry = PageEntry {
                id: -(tab.tab_index as i64), // negative id for non-indexed
                name: tab_title.clone(),
                url: tab.url.clone(),
                description: String::new(),
                pinyin_full: String::new(),
                pinyin_initials: String::new(),
                created_at: String::new(),
                updated_at: String::new(),
            };
            results.push(SearchResult {
                entry: fake_entry,
                score: max_score,
                is_open: true,
                source: "open_tab".into(),
            });
        }
    }

    // Sort by score descending, but boost open tabs slightly
    results.sort_by(|a, b| {
        let a_boost = if a.is_open { 100 } else { 0 };
        let b_boost = if b.is_open { 100 } else { 0 };
        (b.score + b_boost).cmp(&(a.score + a_boost))
    });

    Ok(results)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInput {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
}

#[tauri::command]
pub fn add_page(
    input: PageInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<PageEntry, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    let (pinyin_full, pinyin_initials) = SearchIndex::compute_pinyin(&input.name);
    let desc = input.description.unwrap_or_default();

    let id = queries::insert_page(
        &app.db,
        &input.name,
        &input.url,
        &desc,
        &pinyin_full,
        &pinyin_initials,
    )
    .map_err(|e| e.to_string())?;

    queries::get_page_by_id(&app.db, id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to retrieve created page".into())
}

#[tauri::command]
pub fn update_page(
    id: i64,
    input: PageInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<PageEntry, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    let (pinyin_full, pinyin_initials) = SearchIndex::compute_pinyin(&input.name);
    let desc = input.description.unwrap_or_default();

    queries::update_page(&app.db, id, &input.name, &input.url, &desc, &pinyin_full, &pinyin_initials)
        .map_err(|e| e.to_string())?;

    queries::get_page_by_id(&app.db, id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to retrieve updated page".into())
}

#[tauri::command]
pub fn delete_page(
    id: i64,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    queries::delete_page(&app.db, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clean_duplicate_pages(
    state: State<'_, Mutex<AppState>>,
) -> Result<usize, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    queries::clean_duplicates(&app.db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_pages(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PageEntry>, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    queries::get_all_pages(&app.db).map_err(|e| e.to_string())
}

// --- helpers ---

fn get_open_tabs_silent(browser_key: &str) -> Option<Vec<OpenTab>> {
    let browser_type = match browser_key {
        "edge" => BrowserType::Edge,
        "firefox" => BrowserType::Firefox,
        _ => BrowserType::Chrome,
    };
    let browser = browsers::create_browser(&browser_type).ok()?;
    browser.get_open_tabs().ok()
}

fn normalize_url(url: &str) -> String {
    url.trim_end_matches('/').to_lowercase()
}

fn is_url_open(url: &str, open_tabs: &[OpenTab]) -> bool {
    let norm = normalize_url(url);
    open_tabs.iter().any(|t| {
        let tab_url = normalize_url(&t.url);
        tab_url == norm || tab_url.starts_with(&norm) || norm.starts_with(&tab_url)
    })
}
