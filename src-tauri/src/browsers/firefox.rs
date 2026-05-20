use super::{Browser, BrowserType, OpenTab};

pub struct FirefoxBrowser;

impl FirefoxBrowser {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

pub fn is_installed() -> bool {
    std::path::Path::new("/Applications/Firefox.app").exists()
}

impl Browser for FirefoxBrowser {
    fn get_open_tabs(&self) -> Result<Vec<OpenTab>, String> {
        // TODO: Firefox AppleScript support
        Ok(Vec::new())
    }

    fn activate_tab(&self, _tab: &OpenTab) -> Result<(), String> {
        Err("Firefox tab activation not yet supported".into())
    }

    fn open_new_tab(&self, url: &str) -> Result<(), String> {
        let _ = std::process::Command::new("open")
            .arg("-a")
            .arg("Firefox")
            .arg(url)
            .output();
        Ok(())
    }

    fn close_tab(&self, _tab: &OpenTab) -> Result<(), String> {
        Err("Firefox tab closing not yet supported".into())
    }

    fn browser_type(&self) -> BrowserType {
        BrowserType::Firefox
    }
}
