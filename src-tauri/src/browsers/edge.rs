use super::{Browser, BrowserType, OpenTab};
use std::process::Command;
use std::time::Duration;
use std::thread;

pub struct EdgeBrowser;

impl EdgeBrowser {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

pub fn is_installed() -> bool {
    std::path::Path::new("/Applications/Microsoft Edge.app").exists()
}

fn is_running() -> bool {
    let script = r#"
        tell application "System Events"
            return (name of processes) contains "Microsoft Edge"
        end tell
    "#;
    run_osascript(script)
        .map(|out| out.trim().to_lowercase() == "true")
        .unwrap_or(false)
}

fn ensure_edge_running() -> Result<(), String> {
    if is_running() {
        return Ok(());
    }
    let _ = Command::new("open")
        .arg("-a")
        .arg("Microsoft Edge")
        .output()
        .map_err(|e| format!("Failed to launch Edge: {}", e))?;
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(500));
        if is_running() {
            thread::sleep(Duration::from_millis(500));
            return Ok(());
        }
    }
    Err("Edge failed to start in time".into())
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

impl Browser for EdgeBrowser {
    fn get_open_tabs(&self) -> Result<Vec<OpenTab>, String> {
        if !is_running() {
            return Ok(Vec::new());
        }
        let script = r#"
            tell application "Microsoft Edge"
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
        super::chrome::parse_tabs_output(&output)
    }

    fn activate_tab(&self, tab: &OpenTab) -> Result<(), String> {
        ensure_edge_running()?;
        let script = format!(
            r#"tell application "Microsoft Edge"
                activate
                tell window {}
                    set active tab index to {}
                    set index to 1
                end tell
            end tell"#,
            tab.window_id, tab.tab_index
        );
        run_osascript(&script)?;
        Ok(())
    }

    fn open_new_tab(&self, url: &str) -> Result<(), String> {
        ensure_edge_running()?;
        let url = ensure_protocol(url);
        let escaped_url = url.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            r#"tell application "Microsoft Edge"
                activate
                if (count of windows) > 0 then
                    tell window 1
                        make new tab with properties {{URL:"{}"}}
                    end tell
                else
                    make new window with properties {{tab URL:"{}"}}
                end if
            end tell"#,
            escaped_url, escaped_url
        );
        run_osascript(&script)?;
        Ok(())
    }

    fn close_tab(&self, tab: &OpenTab) -> Result<(), String> {
        if !is_running() {
            return Ok(());
        }
        let escaped_url = tab.url.replace('"', "\\\"");
        let script = format!(
            r#"tell application "Microsoft Edge"
                repeat with w in every window
                    repeat with t in every tab of w
                        if URL of t starts with "{}" then
                            close t
                            return "closed"
                        end if
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
        BrowserType::Edge
    }
}
