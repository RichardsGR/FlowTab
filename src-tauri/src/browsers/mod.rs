pub mod chrome;
pub mod firefox;
pub mod safari;
pub mod bookmarks;
pub mod history;
pub mod macos;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrowserType {
    Chrome,
    Safari,
    Firefox,
}

impl BrowserType {
    pub fn from_key(key: &str) -> Self {
        match key {
            "safari" => Self::Safari,
            "firefox" => Self::Firefox,
            _ => Self::Chrome,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Chrome => "Chrome",
            Self::Safari => "Safari",
            Self::Firefox => "Firefox",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTab {
    pub url: String,
    pub title: String,
    pub window_id: usize,
    pub tab_index: usize,
}

pub trait Browser {
    fn get_open_tabs(&self) -> Result<Vec<OpenTab>, String>;
    fn activate_tab(&self, tab: &OpenTab) -> Result<(), String>;
    fn open_new_tab(&self, url: &str) -> Result<(), String>;
    fn close_tab(&self, tab: &OpenTab) -> Result<(), String>;
    fn browser_type(&self) -> BrowserType;
}

/// Detect which browsers are installed on the system
pub fn detect_installed_browsers() -> Vec<BrowserType> {
    let mut browsers = Vec::new();

    if chrome::is_installed() {
        browsers.push(BrowserType::Chrome);
    }
    if safari::is_installed() {
        browsers.push(BrowserType::Safari);
    }
    if firefox::is_installed() {
        browsers.push(BrowserType::Firefox);
    }

    browsers
}

/// Create a browser handler for the given browser type
pub fn create_browser(browser_type: &BrowserType) -> Result<Box<dyn Browser + Send>, String> {
    match browser_type {
        BrowserType::Chrome => Ok(Box::new(chrome::ChromeBrowser::new()?)),
        BrowserType::Safari => Ok(Box::new(safari::SafariBrowser::new()?)),
        BrowserType::Firefox => Ok(Box::new(firefox::FirefoxBrowser::new()?)),
    }
}

/// Parse tab output. Each line is:
/// `window_index \t tab_index \t url \t title`
pub(crate) fn parse_tabs_output(output: &str) -> Result<Vec<OpenTab>, String> {
    let mut tabs = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() != 4 {
            continue;
        }

        let window_id = match parts[0].trim().parse::<usize>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let tab_index = match parts[1].trim().parse::<usize>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let url = parts[2].trim();
        if url.is_empty() {
            continue;
        }

        tabs.push(OpenTab {
            url: url.to_string(),
            title: parts[3].trim().to_string(),
            window_id,
            tab_index,
        });
    }

    Ok(tabs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_window_and_tab_indices_per_window() {
        let input = "1\t1\thttps://a.com\tA\n1\t2\thttps://b.com\tB\n2\t1\thttps://c.com\tC\n";
        let tabs = parse_tabs_output(input).unwrap();
        assert_eq!(tabs.len(), 3);
        assert_eq!((tabs[0].window_id, tabs[0].tab_index), (1, 1));
        assert_eq!((tabs[1].window_id, tabs[1].tab_index), (1, 2));
        assert_eq!((tabs[2].window_id, tabs[2].tab_index), (2, 1));
    }

    #[test]
    fn skips_malformed_lines_and_empty_urls() {
        let input = "not-a-tab\n1\t1\t\tEmpty\n1\t2\thttps://ok.com\tOK\n";
        let tabs = parse_tabs_output(input).unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].url, "https://ok.com");
    }

    #[test]
    fn from_key_maps_supported_browsers() {
        assert_eq!(BrowserType::from_key("chrome"), BrowserType::Chrome);
        assert_eq!(BrowserType::from_key("safari"), BrowserType::Safari);
        assert_eq!(BrowserType::from_key("firefox"), BrowserType::Firefox);
    }

    #[test]
    fn from_key_falls_back_to_chrome_for_legacy_and_unknown_values() {
        assert_eq!(BrowserType::from_key("edge"), BrowserType::Chrome);
        assert_eq!(BrowserType::from_key(""), BrowserType::Chrome);
        assert_eq!(BrowserType::from_key("whatever"), BrowserType::Chrome);
    }
}
