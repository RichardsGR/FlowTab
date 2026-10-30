# Safari 支持、Edge 移除与 Chrome 多窗口修复

- 日期：2026-10-30
- 分支：feature/tab-actions
- 状态：待评审

## 目标

1. 浏览器控制新增 Safari。
2. 彻底移除 Edge 支持（控制、书签导入、历史分析、文档）。
3. 修复 Chrome 多窗口下标签索引错误，使多窗口切换正确。

## 非目标

- Firefox 的 AppleScript 支持（保持现有存根：只能新建标签页）。
- 同时控制多个浏览器。
- Windows / Linux 的浏览器控制。

## 配置项

沿用现有设置项 `active_browser`（“默认浏览器”），单选：

| 值 | 含义 |
|---|---|
| `chrome` | 控制 Chrome |
| `safari` | 控制 Safari |
| `firefox` | 控制 Firefox（仅新建标签页） |

默认仍为 `chrome`。启动时做一次迁移：`active_browser = 'edge'` 改为 `'chrome'`。

## 后端设计

### BrowserType

```rust
pub enum BrowserType { Chrome, Safari, Firefox }

impl BrowserType {
    pub fn from_key(key: &str) -> Self {
        match key {
            "safari" => Self::Safari,
            "firefox" => Self::Firefox,
            _ => Self::Chrome,
        }
    }
    pub fn display_name(&self) -> &'static str { /* "Chrome" / "Safari" / "Firefox" */ }
}
```

`from_key` 替换 `page_commands.rs`、`browser_commands.rs` 中重复的 `match browser_key`。

### Safari（新增 `src-tauri/src/browsers/safari.rs`）

实现 `Browser` trait：

- `is_installed`：`/Applications/Safari.app` 存在（macOS 恒装）。
- `is_running`：`pgrep -qx Safari`。
- `get_open_tabs`：AppleScript 遍历 `windows → tabs`，每行输出
  `窗口索引 \t 标签索引 \t URL \t 标题`；窗口内标签索引独立计算，过滤空 URL。
- `activate_tab`：`set current tab to tab N of window W`，并将该窗口置前。
- `open_new_tab`：`open -a Safari <url>`（未运行时自动启动）。
- `close_tab`：按 URL 前缀匹配，`close t` 关闭。

AppleScript 参数中的 URL 需要转义反斜杠与双引号。可用性依赖 macOS 自动化授权。

### Chrome 多窗口修复

现有脚本把所有标签的 `window_id` 写死为 1、`tab_index` 跨窗口累加，导致第二个窗口的标签切错。

修复：Chrome 与 Safari 共用同一个标签页输出格式与解析函数 `parse_tabs_output`，从中解析真实的窗口索引与窗口内标签索引。`parse_tabs_output` 移到 `browsers/mod.rs`。

### 自动化权限错误

`osascript` 失败时识别以下特征，统一映射为 `AUTOMATION_DENIED:<浏览器名>`：

- 错误码 `-1743`
- `Not authorized to send Apple events`
- `not allowed to send Apple events`

Chrome 与 Safari 的脚本执行都应用该映射；其他错误原样返回。

### Edge 移除清单

- 删除 `src-tauri/src/browsers/edge.rs`。
- `browsers/mod.rs`：移除 `mod edge`、`BrowserType::Edge`、工厂分支、安装检测。
- `bookmarks.rs`：移除 Edge 书签路径。
- `history.rs`：移除 Edge 历史路径。
- 所有 `match` / `if` 中的 Edge 分支。
- 前端浏览器下拉移除 Edge 选项。
- `README.md`、`TECHNICAL.md`：更新浏览器支持表与说明，删除 Edge 相关描述。

### 单元测试

- `BrowserType::from_key`：chrome / safari / firefox / edge / 未知值。
- `parse_tabs_output`：多窗口、多标签索引正确；空行与空 URL 过滤。
- 权限错误识别：`-1743`、大小写不同的 `Not authorized...`、普通错误。

## 前端设计

### 设置面板

- “默认浏览器”下拉：Chrome / Safari / Firefox。
- 选中 Safari 时显示提示：
  “首次使用需在 系统设置 → 隐私与安全性 → 自动化 中允许 FlowTab 控制 Safari。”
- 读取到非法/旧值（如 `edge`）时回退到 `chrome`。

### 搜索浮窗

- 新增错误横幅状态：捕获 `switch_or_open_tab` / `close_tab` 返回的
  `AUTOMATION_DENIED:<browser>`，显示授权指引；其他错误仍写 console。
- 搜索本身不受影响：权限缺失时仍返回索引结果，只是没有“已打开”标记。
- 权限被拒绝时不自动回退新开标签页。

### i18n

新增中英文 key：

- `safari_first_use_hint`
- `automation_denied`（含 `{browser}` 占位符）

## 验证

1. `cargo test`（新增单测通过）。
2. `cargo check`。
3. `npm run build`。
4. 手动验证：
   - 设置中选择 Safari 后重启，设置持久化。
   - Safari 首次操作触发系统授权；允许后可列标签、切换、复制、关闭。
   - 拒绝授权时搜索正常、操作显示提示横幅。
   - Safari 未运行时 Enter 自动启动并打开页面。
   - Chrome 多窗口下切换正确切到目标窗口的标签页。
   - 书签导入不再读取 Edge，其他浏览器不受影响。

## 风险

- 开发构建签名变化可能导致 macOS 每次重新询问自动化权限；正式签名应用只需一次。
- Safari 标签标题可能含制表符/换行，解析时跳过异常行。
- 历史数据库可能残留已导入的 Edge 条目，属于用户数据，不主动删除。
