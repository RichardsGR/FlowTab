use super::{macos, parse_tabs_output, Browser, BrowserType, OpenTab};

pub struct SafariBrowser;

impl SafariBrowser {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

pub fn is_installed() -> bool {
    ["/Applications/Safari.app", "/System/Applications/Safari.app"]
        .iter()
        .any(|p| std::path::Path::new(p).exists())
}

fn is_running() -> bool {
    macos::is_process_running("Safari")
}

impl Browser for SafariBrowser {
    fn get_open_tabs(&self) -> Result<Vec<OpenTab>, String> {
        if !is_running() {
            return Ok(Vec::new());
        }

        // Note: `tab` inside a tell block resolves to the app's `tab` class,
        // so use explicit ASCII characters for field/newline separators.
        let script = r#"
            set sep to ASCII character 9
            tell application "Safari"
                set output to ""
                repeat with w in windows
                    set widx to index of w
                    set tidx to 0
                    repeat with t in tabs of w
                        set tidx to tidx + 1
                        set output to output & widx & sep & tidx & sep & (URL of t) & sep & (name of t) & (ASCII character 10)
                    end repeat
                end repeat
                return output
            end tell
        "#;

        let output = macos::run_osascript(script, BrowserType::Safari.display_name())?;
        parse_tabs_output(&output)
    }

    fn activate_tab(&self, tab: &OpenTab) -> Result<(), String> {
        if !is_running() {
            return macos::open_url("Safari", &tab.url);
        }

        let script = format!(
            r#"tell application "Safari"
                activate
                set current tab of window {} to tab {} of window {}
                set index of window {} to 1
            end tell"#,
            tab.window_id, tab.tab_index, tab.window_id, tab.window_id
        );

        match macos::run_osascript(&script, BrowserType::Safari.display_name()) {
            Ok(_) => Ok(()),
            Err(e) if e.starts_with("AUTOMATION_DENIED") => Err(e),
            Err(_) => macos::open_url("Safari", &tab.url),
        }
    }

    fn open_new_tab(&self, url: &str) -> Result<(), String> {
        macos::open_url("Safari", url)
    }

    fn close_tab(&self, tab: &OpenTab) -> Result<(), String> {
        if !is_running() {
            return Ok(());
        }

        let escaped_url = tab.url.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            r#"tell application "Safari"
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

        let result = macos::run_osascript(&script, BrowserType::Safari.display_name())?;
        if result == "closed" {
            Ok(())
        } else {
            Err("Tab not found".into())
        }
    }

    fn browser_type(&self) -> BrowserType {
        BrowserType::Safari
    }
}
