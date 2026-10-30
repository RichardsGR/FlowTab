use super::{macos, parse_tabs_output, Browser, BrowserType, OpenTab};

pub struct ChromeBrowser;

impl ChromeBrowser {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

pub fn is_installed() -> bool {
    let paths = [
        "/Applications/Google Chrome.app",
        "/Applications/Chrome.app",
    ];
    paths.iter().any(|p| std::path::Path::new(p).exists())
}

fn is_running() -> bool {
    macos::is_process_running("Google Chrome")
}

impl Browser for ChromeBrowser {
    fn get_open_tabs(&self) -> Result<Vec<OpenTab>, String> {
        if !is_running() {
            return Ok(Vec::new());
        }

        let script = r#"
            tell application "Google Chrome"
                set output to ""
                repeat with w in windows
                    set widx to index of w
                    repeat with t in tabs of w
                        set tidx to index of t
                        set output to output & widx & tab & tidx & tab & (URL of t) & tab & (title of t) & linefeed
                    end repeat
                end repeat
                return output
            end tell
        "#;

        let output = macos::run_osascript(script, BrowserType::Chrome.display_name())?;
        parse_tabs_output(&output)
    }

    fn activate_tab(&self, tab: &OpenTab) -> Result<(), String> {
        if !is_running() {
            return macos::open_url("Google Chrome", &tab.url);
        }

        let script = format!(
            r#"tell application "Google Chrome"
                activate
                tell window {}
                    set active tab index to {}
                    set index to 1
                end tell
            end tell"#,
            tab.window_id, tab.tab_index
        );

        match macos::run_osascript(&script, BrowserType::Chrome.display_name()) {
            Ok(_) => Ok(()),
            Err(e) if e.starts_with("AUTOMATION_DENIED") => Err(e),
            Err(_) => macos::open_url("Google Chrome", &tab.url),
        }
    }

    fn open_new_tab(&self, url: &str) -> Result<(), String> {
        macos::open_url("Google Chrome", url)
    }

    fn close_tab(&self, tab: &OpenTab) -> Result<(), String> {
        if !is_running() {
            return Ok(());
        }

        let escaped_url = tab.url.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            r#"tell application "Google Chrome"
                repeat with w in windows
                    repeat with t in tabs of w
                        if URL of t starts with "{}" then
                            close t
                            return "closed"
                        end if
                    end repeat
                end repeat
                return "not_found"
            end tell"#,
            escaped_url
        );

        let result = macos::run_osascript(&script, BrowserType::Chrome.display_name())?;
        if result == "closed" {
            Ok(())
        } else {
            Err("Tab not found".into())
        }
    }

    fn browser_type(&self) -> BrowserType {
        BrowserType::Chrome
    }
}
