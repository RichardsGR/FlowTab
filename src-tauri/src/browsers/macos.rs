use std::process::Command;

pub(crate) fn is_process_running(process_name: &str) -> bool {
    Command::new("pgrep")
        .arg("-qx")
        .arg(process_name)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub(crate) fn open_url(app_name: &str, url: &str) -> Result<(), String> {
    let url = ensure_protocol(url);
    let status = Command::new("open")
        .arg("-a")
        .arg(app_name)
        .arg(&url)
        .status()
        .map_err(|e| format!("Failed to launch {}: {}", app_name, e))?;

    if !status.success() {
        return Err(format!("open command failed for {}", app_name));
    }
    Ok(())
}

pub(crate) fn run_osascript(script: &str, browser_name: &str) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(map_script_error(&stderr, browser_name));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn map_script_error(stderr: &str, browser_name: &str) -> String {
    let lower = stderr.to_lowercase();
    if stderr.contains("-1743")
        || lower.contains("not authorized to send apple events")
        || lower.contains("not allowed to send apple events")
    {
        return format!("AUTOMATION_DENIED:{}", browser_name);
    }
    format!("osascript error: {}", stderr.trim())
}

fn ensure_protocol(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_not_authorized_error_code() {
        let stderr = "execution error: Not authorized to send Apple events to Safari. (-1743)";
        assert_eq!(map_script_error(stderr, "Safari"), "AUTOMATION_DENIED:Safari");
    }

    #[test]
    fn maps_not_authorized_message_case_insensitively() {
        let stderr = "execution error: NOT AUTHORIZED TO SEND APPLE EVENTS to Chrome.";
        assert_eq!(map_script_error(stderr, "Chrome"), "AUTOMATION_DENIED:Chrome");
    }

    #[test]
    fn keeps_other_errors() {
        let stderr = "execution error: window 9 doesn't understand the message (-1728)";
        assert!(map_script_error(stderr, "Safari").starts_with("osascript error:"));
    }
}
