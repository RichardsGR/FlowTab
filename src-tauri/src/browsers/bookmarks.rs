use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BookmarkItem {
    pub name: String,
    pub url: String,
}

/// Parse Chrome bookmarks JSON file
/// Chrome stores bookmarks in a nested JSON structure with "roots" -> folders -> children
pub fn parse_chrome_bookmarks(path: &str) -> Result<Vec<BookmarkItem>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read bookmarks file: {}", e))?;

    let root: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse bookmarks JSON: {}", e))?;

    let mut items = Vec::new();
    flatten_bookmarks_node(&root["roots"], &mut items);
    Ok(items)
}

/// Parse Firefox bookmarks from places.sqlite
pub fn parse_firefox_bookmarks(_path: &str) -> Result<Vec<BookmarkItem>, String> {
    // Firefox uses a custom SQLite schema in places.sqlite
    // TODO: Implement when Firefox support is added
    Ok(Vec::new())
}

fn flatten_bookmarks_node(node: &Value, items: &mut Vec<BookmarkItem>) {
    match node {
        Value::Object(obj) => {
            // If it has a URL, it's a bookmark
            if let Some(url) = obj.get("url").and_then(|v| v.as_str()) {
                if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                    if url.starts_with("http") {
                        items.push(BookmarkItem {
                            name: name.to_string(),
                            url: url.to_string(),
                        });
                    }
                }
            }

            // Recurse into children
            if let Some(children) = obj.get("children").and_then(|v| v.as_array()) {
                for child in children {
                    flatten_bookmarks_node(child, items);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                flatten_bookmarks_node(item, items);
            }
        }
        _ => {}
    }
}

/// Get the default bookmarks paths for installed browsers
pub fn get_bookmark_paths() -> Vec<(super::BrowserType, PathBuf)> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut paths = Vec::new();

    // Chrome
    let chrome_path = PathBuf::from(format!(
        "{}/Library/Application Support/Google/Chrome/Default/Bookmarks",
        home
    ));
    if chrome_path.exists() {
        paths.push((super::BrowserType::Chrome, chrome_path));
    }

    // Firefox
    let firefox_dir = PathBuf::from(format!(
        "{}/Library/Application Support/Firefox/Profiles",
        home
    ));
    if firefox_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&firefox_dir) {
            for entry in entries.flatten() {
                let places = entry.path().join("places.sqlite");
                if places.exists() {
                    paths.push((super::BrowserType::Firefox, places));
                    break;
                }
            }
        }
    }

    paths
}
