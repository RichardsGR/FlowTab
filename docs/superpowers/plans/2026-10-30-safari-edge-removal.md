# Safari 支持、Edge 移除与 Chrome 多窗口修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 FlowTab 增加 Safari 浏览器控制，彻底移除 Edge，并修复 Chrome 多窗口标签索引错误。

**Architecture:** 沿用现有的 `Browser` trait 与 `osascript`/`open` 命令模式。新增 `safari.rs` 实现 Safari；抽出 `macos.rs` 统一脚本执行、权限错误映射与打开 URL；Chrome/Safari 共用带窗口索引的标签页解析格式。前端沿用现有 `active_browser` 单选设置，并在搜索浮窗显示自动化权限提示。

**Tech Stack:** Rust 2021 / Tauri 2 / rusqlite / Svelte 5 / TypeScript / Vite 6 / macOS AppleScript

## Global Constraints

- 仅目标 macOS；不新增 Rust 或 npm 依赖。
- 浏览器配置项 `active_browser` 为单选：`chrome` / `safari` / `firefox`；默认 `chrome`；旧值 `edge` 在启动时迁移为 `chrome`。
- 权限错误统一格式：`AUTOMATION_DENIED:<浏览器名>`（如 `AUTOMATION_DENIED:Safari`）。
- 权限被拒绝时不得自动回退为新开标签页。
- Safari 与 Chrome 的标签页输出统一为每行 `窗口索引 \t 标签索引 \t URL \t 标题`。
- 工作区已有未提交的 i18n 改造（`src/tauri/i18n.ts` 等），本计划在其之上叠加，不得回退或覆盖。
- **提交策略：未获用户明确指示前不要执行 `git commit` / `git push`。每个任务结束时改动留在工作区。**
- 每个任务结束后按任务内写明的命令做编译/测试验证。

---

### Task 1: 彻底移除 Edge

**Files:**
- Delete: `src-tauri/src/browsers/edge.rs`
- Modify: `src-tauri/src/browsers/mod.rs`
- Modify: `src-tauri/src/browsers/bookmarks.rs`
- Modify: `src-tauri/src/browsers/history.rs`
- Modify: `src-tauri/src/commands/page_commands.rs`
- Modify: `src-tauri/src/commands/browser_commands.rs`
- Modify: `src-tauri/src/db/init.rs`
- Modify: `src/lib/SettingsPanel.svelte`
- Modify: `README.md`
- Modify: `TECHNICAL.md`

**Interfaces:**
- Consumes: 现有 `BrowserType::{Chrome, Firefox}`、`Browser` trait。
- Produces: 不含 `Edge` 的 `BrowserType`；`get_bookmark_paths()` / `get_history_paths()` 不再返回 Edge；启动时把 `active_browser='edge'` 迁移为 `chrome`。

- [ ] **Step 1: 删除 Edge 模块文件**

```bash
rm src-tauri/src/browsers/edge.rs
```

- [ ] **Step 2: 从 `browsers/mod.rs` 移除 Edge**

把文件开头与工厂/检测部分改成：

```rust
pub mod chrome;
pub mod firefox;
pub mod bookmarks;
pub mod history;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrowserType {
    Chrome,
    Firefox,
}
```

```rust
pub fn detect_installed_browsers() -> Vec<BrowserType> {
    let mut browsers = Vec::new();

    if chrome::is_installed() {
        browsers.push(BrowserType::Chrome);
    }
    if firefox::is_installed() {
        browsers.push(BrowserType::Firefox);
    }

    browsers
}

pub fn create_browser(browser_type: &BrowserType) -> Result<Box<dyn Browser + Send>, String> {
    match browser_type {
        BrowserType::Chrome => Ok(Box::new(chrome::ChromeBrowser::new()?)),
        BrowserType::Firefox => Ok(Box::new(firefox::FirefoxBrowser::new()?)),
    }
}
```

- [ ] **Step 3: 移除 Edge 书签路径（`bookmarks.rs`）**

删除 `get_bookmark_paths()` 中的整个 Edge 块：

```rust
    // Edge
    let edge_path = PathBuf::from(format!(
        "{}/Library/Application Support/Microsoft Edge/Default/Bookmarks",
        home
    ));
    if edge_path.exists() {
        paths.push((super::BrowserType::Edge, edge_path));
    }
```

- [ ] **Step 4: 移除 Edge 历史路径（`history.rs`）**

删除 `get_history_paths()` 中的整个 Edge 块：

```rust
    // Edge
    let edge = PathBuf::from(format!(
        "{}/Library/Application Support/Microsoft Edge/Default/History",
        home
    ));
    if edge.exists() {
        paths.push(edge);
    }
```

- [ ] **Step 5: 更新后端命令中的浏览器匹配**

`src-tauri/src/commands/page_commands.rs` 的 `get_open_tabs_silent`：

```rust
fn get_open_tabs_silent(browser_key: &str) -> Option<Vec<OpenTab>> {
    let browser_type = match browser_key {
        "firefox" => BrowserType::Firefox,
        _ => BrowserType::Chrome,
    };
    let browser = browsers::create_browser(&browser_type).ok()?;
    browser.get_open_tabs().ok()
}
```

`src-tauri/src/commands/browser_commands.rs` 中共有 3 处相同匹配（`switch_or_open_tab`、`open_new_tab`、`close_tab`），全部改为：

```rust
    let browser_type = match browser_key.as_str() {
        "firefox" => BrowserType::Firefox,
        _ => BrowserType::Chrome,
    };
```

- [ ] **Step 6: 数据库迁移旧设置值**

在 `src-tauri/src/db/init.rs` 的 `execute_batch` 末尾追加新设置与迁移（保留已有全部 SQL 不变）：

```sql
        INSERT OR IGNORE INTO settings (key, value) VALUES ('language', 'en');
        UPDATE settings SET value = 'chrome' WHERE key = 'active_browser' AND value = 'edge';
```

- [ ] **Step 7: 前端下拉移除 Edge 并校验旧值**

`src/lib/SettingsPanel.svelte` 的 `loadSettings()` 中，把读取 `active_browser` 的一行改为：

```ts
      const savedBrowser = (await getSetting("active_browser")) ?? "chrome";
      activeBrowser = ["chrome", "firefox"].includes(savedBrowser) ? savedBrowser : "chrome";
```

下拉选项改为：

```svelte
    <select bind:value={activeBrowser}>
      <option value="chrome">Google Chrome</option>
      <option value="firefox">Firefox</option>
    </select>
```

- [ ] **Step 8: 更新文档中的 Edge 描述**

`README.md`：

- `一键导入 Chrome/Edge 浏览器书签` → `一键导入 Chrome 浏览器书签`
- README 中不再出现 `Edge` 字样。

`TECHNICAL.md`：

- 项目结构树中删除 `│   │   ├── edge.rs            # Edge 支持`。
- 浏览器通信表中删除 Edge 相关描述。
- 书签/历史读取描述中删除 `Edge`（Chrome/Edge/Arc/Brave → Chrome/Arc/Brave）。

- [ ] **Step 9: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
rg -n "Edge|edge" src src-tauri/src README.md TECHNICAL.md
```

预期：编译与前端构建通过；`rg` 无输出（`edge` 作为 `edge case` 等英文单词也不应出现在项目源码/文档中）。

---

### Task 2: Chrome 多窗口修复 + 共享解析器 + macOS 辅助模块

**Files:**
- Create: `src-tauri/src/browsers/macos.rs`
- Modify: `src-tauri/src/browsers/mod.rs`
- Modify: `src-tauri/src/browsers/chrome.rs`
- Test: `src-tauri/src/browsers/mod.rs`、`src-tauri/src/browsers/macos.rs` 内联 `#[cfg(test)]`

**Interfaces:**
- Produces:
  - `pub(crate) fn parse_tabs_output(output: &str) -> Result<Vec<OpenTab>, String>`（`browsers/mod.rs`）
  - `pub(crate) fn is_process_running(name: &str) -> bool`
  - `pub(crate) fn open_url(app_name: &str, url: &str) -> Result<(), String>`
  - `pub(crate) fn run_osascript(script: &str, browser_name: &str) -> Result<String, String>`
  - `pub(crate) fn map_script_error(stderr: &str, browser_name: &str) -> String`
- Consumes: `OpenTab { url, title, window_id, tab_index }`、`BrowserType::display_name()`（Task 3 提供；本任务先使用字面量 `"Chrome"`，Task 3 再切换为 `display_name()`）。

- [ ] **Step 1: 在 `browsers/mod.rs` 写失败的解析器测试**

在文件末尾添加：

```rust
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
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml parses_window_and_tab_indices_per_window
```

预期：编译失败，提示找不到 `parse_tabs_output`。

- [ ] **Step 3: 在 `browsers/mod.rs` 实现解析器**

在 `create_browser` 之后添加：

```rust
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
```

- [ ] **Step 4: 运行解析器测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

预期：两个解析器测试通过。

- [ ] **Step 5: 新建 `macos.rs` 并写失败的权限映射测试**

先创建：

```rust
use std::process::Command;

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
```

- [ ] **Step 6: 运行测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml maps_not_authorized_error_code
```

预期：编译失败，提示找不到 `map_script_error`。

- [ ] **Step 7: 实现 `macos.rs` 全部辅助函数**

把文件内容替换为：

```rust
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
```

- [ ] **Step 8: 在 `mod.rs` 注册模块**

```rust
pub mod macos;
```

- [ ] **Step 9: 用共享辅助函数重写 `chrome.rs`**

把文件内容替换为：

```rust
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

        let output = macos::run_osascript(script, "Chrome")?;
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

        match macos::run_osascript(&script, "Chrome") {
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

        let result = macos::run_osascript(&script, "Chrome")?;
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
```

> 说明：`display_name()` 在 Task 3 才加入，本任务先使用字面量 `"Chrome"`；Task 3 Step 6 会统一替换为 `BrowserType::Chrome.display_name()`。

- [ ] **Step 10: 验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

预期：全部单元测试通过，编译无错误。

---

### Task 3: Safari 后端 + `from_key` / `display_name`

**Files:**
- Create: `src-tauri/src/browsers/safari.rs`
- Modify: `src-tauri/src/browsers/mod.rs`
- Modify: `src-tauri/src/commands/page_commands.rs`
- Modify: `src-tauri/src/commands/browser_commands.rs`
- Modify: `src-tauri/src/browsers/chrome.rs`（替换字面量为 `display_name()`）
- Modify: `README.md`
- Modify: `TECHNICAL.md`
- Test: `src-tauri/src/browsers/mod.rs` 内联 `#[cfg(test)]`

**Interfaces:**
- Consumes: Task 2 的 `macos::{run_osascript, open_url, is_process_running}`、`parse_tabs_output`。
- Produces:
  - `BrowserType::{Chrome, Safari, Firefox}`
  - `BrowserType::from_key(key: &str) -> BrowserType`
  - `BrowserType::display_name(&self) -> &'static str`
  - `SafariBrowser` 实现 `Browser`

- [ ] **Step 1: 写失败的 `from_key` 测试**

在 `browsers/mod.rs` 的 `#[cfg(test)] mod tests` 中加入：

```rust
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
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml from_key_maps_supported_browsers
```

预期：编译失败，提示 `Safari` 变体或 `from_key` 不存在。

- [ ] **Step 3: 扩展 `BrowserType` 并新增 Safari 模块**

`browsers/mod.rs` 顶部与枚举改为：

```rust
pub mod chrome;
pub mod firefox;
pub mod safari;
pub mod macos;
pub mod bookmarks;
pub mod history;
```

```rust
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
```

`detect_installed_browsers` 加入 Safari：

```rust
    if safari::is_installed() {
        browsers.push(BrowserType::Safari);
    }
```

`create_browser` 加入：

```rust
        BrowserType::Safari => Ok(Box::new(safari::SafariBrowser::new()?)),
```

- [ ] **Step 4: 新建 `safari.rs`**

```rust
use super::{macos, parse_tabs_output, Browser, BrowserType, OpenTab};

pub struct SafariBrowser;

impl SafariBrowser {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

pub fn is_installed() -> bool {
    std::path::Path::new("/Applications/Safari.app").exists()
}

fn is_running() -> bool {
    macos::is_process_running("Safari")
}

impl Browser for SafariBrowser {
    fn get_open_tabs(&self) -> Result<Vec<OpenTab>, String> {
        if !is_running() {
            return Ok(Vec::new());
        }

        let script = r#"
            tell application "Safari"
                set output to ""
                repeat with w in windows
                    set widx to index of w
                    repeat with t in tabs of w
                        set tidx to index of t
                        set output to output & widx & tab & tidx & tab & (URL of t) & tab & (name of t) & linefeed
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
```

- [ ] **Step 5: 把命令层改为 `from_key`**

`src-tauri/src/commands/page_commands.rs` 的 `get_open_tabs_silent`：

```rust
fn get_open_tabs_silent(browser_key: &str) -> Option<Vec<OpenTab>> {
    let browser_type = BrowserType::from_key(browser_key);
    let browser = browsers::create_browser(&browser_type).ok()?;
    browser.get_open_tabs().ok()
}
```

`src-tauri/src/commands/browser_commands.rs` 中共有 3 处，全部改为：

```rust
    let browser_type = BrowserType::from_key(&browser_key);
```

- [ ] **Step 6: 把 Chrome 中的字面量替换为 `display_name()`**

`src-tauri/src/browsers/chrome.rs` 中所有 `"Chrome"` 作为 `run_osascript` 第二参数的位置，改为：

```rust
BrowserType::Chrome.display_name()
```

- [ ] **Step 7: 运行测试与编译检查**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

预期：`from_key` 两个测试与解析器/权限测试全部通过。

- [ ] **Step 8: 更新文档中的浏览器支持**

`README.md`：

- 功能列表增加：`支持 Chrome / Safari 标签页控制，Firefox 仅支持新建标签页`
- 书签导入描述保持 Chrome（Task 1 已改）；历史分析描述如出现 Edge 已删除。
- “浏览器冷启动”改为：`Chrome / Safari 未运行时自动启动并打开目标页面`

`TECHNICAL.md`：

- 项目结构树加入：
  - `│       │   ├── safari.rs         # Safari 标签页检测/切换/打开`
  - `│       │   ├── macos.rs          # osascript 执行、权限错误映射、open 命令`
- 技术栈表浏览器通信行保持 `AppleScript (osascript) / open 命令`。
- 新增一小节说明：

```markdown
### 浏览器支持

| 浏览器 | 列标签 | 切换 | 新建 | 关闭 |
|--------|--------|------|------|------|
| Chrome | ✅ | ✅ | ✅ | ✅ |
| Safari | ✅ | ✅ | ✅ | ✅ |
| Firefox | ❌ | ❌ | ✅ | ❌ |

Safari 与 Chrome 均通过 AppleScript 操作，首次使用需要在
系统设置 → 隐私与安全性 → 自动化 中允许 FlowTab 控制对应浏览器。
```

---

### Task 4: 设置面板加入 Safari

**Files:**
- Modify: `src/lib/SettingsPanel.svelte`
- Modify: `src/tauri/i18n.ts`

**Interfaces:**
- Consumes: `active_browser` 设置项；`i18n(key, params?)`。
- Produces: 下拉选项 `chrome` / `safari` / `firefox`；选中 Safari 时显示首次授权提示。

- [ ] **Step 1: 添加 i18n key**

在 `src/tauri/i18n.ts` 的 `en` 字典中加入：

```ts
    safari_first_use_hint: "First use requires allowing FlowTab to control Safari in System Settings → Privacy & Security → Automation.",
```

在 `zh` 字典中加入：

```ts
    safari_first_use_hint: "首次使用需在 系统设置 → 隐私与安全性 → 自动化 中允许 FlowTab 控制 Safari。",
```

- [ ] **Step 2: 更新下拉选项与校验**

`src/lib/SettingsPanel.svelte` 的 `loadSettings()`：

```ts
      const savedBrowser = (await getSetting("active_browser")) ?? "chrome";
      activeBrowser = ["chrome", "safari", "firefox"].includes(savedBrowser)
        ? savedBrowser
        : "chrome";
```

下拉：

```svelte
    <select bind:value={activeBrowser}>
      <option value="chrome">Google Chrome</option>
      <option value="safari">Safari</option>
      <option value="firefox">Firefox</option>
    </select>
    {#if activeBrowser === "safari"}
      <p style="font-size:12px;color:var(--text-tertiary);margin-top:4px;">
        {i18n("safari_first_use_hint")}
      </p>
    {/if}
```

- [ ] **Step 3: 验证**

```bash
npm run build
```

预期：Svelte/TS 构建通过。

---

### Task 5: 搜索浮窗权限提示横幅

**Files:**
- Modify: `src/lib/SearchPopup.svelte`
- Modify: `src/styles/search.css`
- Modify: `src/tauri/i18n.ts`

**Interfaces:**
- Consumes: Rust 返回的 `AUTOMATION_DENIED:<browser>`；`i18n(key, params?)`。
- Produces: 搜索浮窗内的 `automation-banner`；操作成功后才隐藏窗口。

- [ ] **Step 1: 添加 i18n key**

`en` 字典：

```ts
    automation_denied: "Permission needed: allow FlowTab to control {browser} in System Settings → Privacy & Security → Automation.",
```

`zh` 字典：

```ts
    automation_denied: "需要授权：请在 系统设置 → 隐私与安全性 → 自动化 中允许 FlowTab 控制 {browser}。",
```

- [ ] **Step 2: 在 `SearchPopup.svelte` 增加状态与错误处理**

在 `handleSelect` 之前加入：

```ts
  let automationError = $state<string | null>(null);

  function handleActionError(e: unknown) {
    const msg = String(e);
    if (msg.includes("AUTOMATION_DENIED")) {
      const browser = msg.split(":")[1]?.trim() || "浏览器";
      automationError = i18n("automation_denied", { browser });
    } else {
      console.error("Browser action failed:", e);
    }
  }
```

把 `handleSelect` 改为“成功后再隐藏”：

```ts
  async function handleSelect(item: SearchResult, forceNew: boolean = false) {
    try {
      if (forceNew) {
        await openNewTab(item.entry.url);
      } else {
        await switchOrOpenTab(item.entry.url, false);
      }
      // Hide only after the browser action succeeds, so permission errors stay visible
      hideWindow().catch(() => {});
    } catch (e) {
      handleActionError(e);
    }
  }
```

新增关闭标签的处理函数：

```ts
  async function handleClose(item: SearchResult) {
    try {
      await closeTab(item.entry.url);
      doSearch(query);
    } catch (e) {
      handleActionError(e);
    }
  }
```

`Tab+Backspace` 分支改为：

```ts
        handleClose(results[selectedIndex]);
```

`ResultList` 的 `onClose` 改为：

```svelte
        onClose={handleClose}
```

- [ ] **Step 3: 添加横幅 DOM**

在 `.search-input-wrapper` 之后、`{#if loading}` 之前插入：

```svelte
    {#if automationError}
      <div class="automation-banner" role="alert">
        <span>{automationError}</span>
        <button
          class="automation-banner-close"
          onclick={() => (automationError = null)}
          aria-label="Dismiss"
        >✕</button>
      </div>
    {/if}
```

- [ ] **Step 4: 添加横幅样式**

在 `src/styles/search.css` 末尾加入：

```css
.automation-banner {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin: 8px 12px 0;
  padding: 8px 10px;
  border: 1px solid rgba(255, 159, 10, 0.45);
  border-radius: 8px;
  background: rgba(255, 159, 10, 0.12);
  color: var(--text-primary);
  font-size: 12px;
  line-height: 1.45;
}

.automation-banner span {
  flex: 1;
}

.automation-banner-close {
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0 2px;
  font-size: 12px;
}
```

- [ ] **Step 5: 验证**

```bash
npm run build
```

预期：构建通过。

---

### Task 6: 全量验证

**Files:** 无新增；验证全部改动。

- [ ] **Step 1: 后端测试与编译**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

预期：全部测试通过，无编译错误。

- [ ] **Step 2: 前端构建**

```bash
npm run build
```

预期：构建成功。

- [ ] **Step 3: 检查残留引用**

```bash
rg -n "Edge|edge|Microsoft Edge" src src-tauri/src README.md TECHNICAL.md
git diff --check
```

预期：`rg` 无 Edge 相关输出；`git diff --check` 无空白错误。

- [ ] **Step 4: 手动验证清单**

运行开发版：

```bash
npm run tauri dev
```

逐项确认：

- [ ] 设置中选择 Safari 并保存，切换视图再回来仍为 Safari。
- [ ] Safari 首次操作弹出自动化授权；允许后可看到“已打开”标记。
- [ ] 在 Safari 中切换标签、`Tab+Enter` 复制、`Tab+⌫` 关闭均生效。
- [ ] 拒绝/撤销授权后，搜索正常返回结果，操作时搜索浮窗保持可见并显示授权提示横幅。
- [ ] Safari 未运行时按 Enter 能自动启动 Safari 并打开页面。
- [ ] Chrome 打开两个窗口，各自多个标签，选择第二个窗口中的标签能正确切到该标签，而不是新开。
- [ ] 书签导入不再读取 Edge；Chrome 书签导入正常。
- [ ] 设置中不再出现 Edge 选项。

- [ ] **Step 5: 汇总改动并等待用户决定是否提交**

```bash
git status -sb
git diff --stat
```

不要自动提交；向用户报告结果并询问是否提交。
