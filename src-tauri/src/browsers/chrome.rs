use super::{Browser, BrowserType, OpenTab};
use std::process::Command;

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
    // Use pgrep to check if Chrome is running (no AppleScript permissions needed)
    Command::new("pgrep")
        .arg("-q")
        .arg("Google Chrome")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

impl Browser for ChromeBrowser {
    fn get_open_tabs(&self) -> Result<Vec<OpenTab>, String> {
        if !is_running() {
            return Ok(Vec::new());
        }

        let script = r#"
            tell application "Google Chrome"
                set output to ""
                set allWindows to every window
                repeat with w in allWindows
                    set allTabs to every tab in w
                    repeat with t in allTabs
                        set tabURL to URL of t
                        set tabTitle to title of t
                        set output to output & tabURL & "|" & tabTitle & "\n"
                    end repeat
                end repeat
                return output
            end tell
        "#;

        let output = run_osascript(script)?;
        parse_tabs_output(&output)
    }

    fn activate_tab(&self, tab: &OpenTab) -> Result<(), String> {
        if !is_running() {
            // Chrome not running, can't switch to tab, just open the URL
            return open_via_shell(&tab.url);
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
        // If AppleScript fails (e.g., window/tab changed), fall back to opening URL
        match run_osascript(&script) {
            Ok(_) => Ok(()),
            Err(_) => open_via_shell(&tab.url),
        }
    }

    fn open_new_tab(&self, url: &str) -> Result<(), String> {
        open_via_shell(url)
    }

    fn close_tab(&self, tab: &OpenTab) -> Result<(), String> {
        if !is_running() {
            return Ok(());
        }
        // Close tab by URL matching (more reliable than index)
        let escaped_url = tab.url.replace('"', "\\\"");
        let script = format!(
            r#"tell application "Google Chrome"
                repeat with w in every window
                    set tabIndex to 1
                    repeat with t in every tab of w
                        if URL of t starts with "{}" then
                            close t
                            return "closed"
                        end if
                        set tabIndex to tabIndex + 1
                    end repeat
                end repeat
            end tell"#,
            escaped_url
        );
        match run_osascript(&script) {
            Ok(r) if r == "closed" => Ok(()),
            Ok(_) => Err("Tab not found".into()),
            Err(e) => Err(e),
        }
    }

    fn browser_type(&self) -> BrowserType {
        BrowserType::Chrome
    }
}

/// Open URL via `open -a "Google Chrome"` — works whether Chrome is running or not
fn open_via_shell(url: &str) -> Result<(), String> {
    let url = ensure_protocol(url);
    let status = Command::new("open")
        .arg("-a")
        .arg("Google Chrome")
        .arg(&url)
        .status()
        .map_err(|e| format!("Failed to launch Chrome: {}", e))?;

    if !status.success() {
        return Err("open command failed".into());
    }
    Ok(())
}

fn ensure_protocol(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}

fn run_osascript(script: &str) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("osascript error: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn parse_tabs_output(output: &str) -> Result<Vec<OpenTab>, String> {
    let mut tabs = Vec::new();
    let window_id: usize = 1;
    let mut last_tab_in_window: usize = 0;

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() >= 2 {
            let url = parts[0].trim().to_string();
            let title = parts[1].trim().to_string();

            tabs.push(OpenTab {
                url,
                title,
                window_id,
                tab_index: last_tab_in_window + 1,
            });
            last_tab_in_window += 1;
        }
    }

    Ok(tabs)
}
