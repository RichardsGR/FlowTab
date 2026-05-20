pub mod chrome;
pub mod firefox;
pub mod edge;
pub mod bookmarks;
pub mod history;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrowserType {
    Chrome,
    Edge,
    Firefox,
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
    if edge::is_installed() {
        browsers.push(BrowserType::Edge);
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
        BrowserType::Edge => Ok(Box::new(edge::EdgeBrowser::new()?)),
        BrowserType::Firefox => Ok(Box::new(firefox::FirefoxBrowser::new()?)),
    }
}
