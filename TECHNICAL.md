# FlowTab 技术实现文档

## 概述

FlowTab 是一款基于 Rust + Tauri 的跨平台浏览器标签页管理桌面应用。通过全局快捷键快速呼出搜索框，实现标签页极速检索、智能去重切换与一键跳转。

## 技术栈

| 层 | 技术 | 版本 |
|---|---|---|
| 桌面框架 | Tauri | 2.x |
| 前端 | Svelte 5 + TypeScript | 5.x |
| 构建 | Vite | 6.x |
| 数据库 | SQLite (rusqlite, bundled) | 0.31 |
| 模糊搜索 | nucleo | 0.5 |
| 拼音转换 | pinyin | 0.11 |
| 浏览器通信 | AppleScript (osascript) / `open` 命令 | macOS 原生 |
| 全局热键 | tauri-plugin-global-shortcut | 2.x |

## 项目结构

```
FlowTab/
├── src-tauri/                     # Rust 后端
│   ├── Cargo.toml                 # Rust 依赖
│   ├── tauri.conf.json            # Tauri 窗口/打包配置
│   ├── capabilities/default.json  # 权限清单
│   ├── icons/                     # 应用图标
│   └── src/
│       ├── main.rs                # 入口
│       ├── lib.rs                 # 模块注册、托盘、IPC 命令
│       ├── state.rs               # AppState (Mutex<Connection>)
│       ├── hotkey.rs              # 全局热键 Ctrl+K
│       ├── db/
│       │   ├── init.rs            # 数据库初始化与迁移
│       │   ├── models.rs          # PageEntry, SearchHistory, AppSettings
│       │   └── queries.rs         # CRUD + URL 去重
│       ├── search/
│       │   ├── fuzzy.rs           # nucleo 模糊匹配 + 拼音转换
│       │   └── index.rs           # 搜索索引（关键词/拼音/URL 加权评分）
│       ├── browsers/
│       │   ├── mod.rs             # Browser trait + BrowserFactory
│       │   ├── chrome.rs          # Chrome 标签页检测/切换/打开
│       │   ├── edge.rs            # Edge 支持
│       │   ├── firefox.rs         # Firefox 存根
│       │   ├── bookmarks.rs       # 书签解析 (JSON)
│       │   └── history.rs         # 浏览记录读取 (SQLite)
│       └── commands/
│           ├── page_commands.rs   # 搜索/CRUD/去重
│           ├── browser_commands.rs# 标签页切换/打开
│           ├── bookmark_commands.rs# 书签导入/历史分析
│           └── settings_commands.rs# 设置读写 + 窗口隐藏
├── src/                           # Svelte 前端
│   ├── main.ts                    # 入口
│   ├── lib/
│   │   ├── App.svelte             # 根组件（视图路由 + 主题初始化）
│   │   ├── SearchPopup.svelte     # 搜索浮窗（输入、结果、键盘导航、失焦关闭）
│   │   ├── ResultList.svelte      # 搜索结果列表
│   │   ├── ResultItem.svelte      # 单条结果（已打开标记）
│   │   ├── EmptyState.svelte      # 空状态
│   │   ├── PageIndexManager.svelte# 页面索引 CRUD 管理
│   │   ├── BookmarkImporter.svelte# 书签导入
│   │   └── SettingsPanel.svelte   # 设置（主题、浏览器、快捷键、历史分析）
│   ├── tauri/
│   │   ├── commands.ts            # 类型化 IPC 调用封装
│   │   └── types.ts               # 共享 TS 类型
│   └── styles/
│       ├── global.css             # CSS 变量（深色/浅色主题）、基础样式
│       └── search.css             # 搜索浮窗、管理页面、设置面板样式
├── index.html
├── package.json
├── vite.config.ts
├── svelte.config.js
├── tsconfig.json
└── design.md                      # 原始需求文档
```

## 数据库 Schema

```sql
-- 页面索引表
CREATE TABLE page_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                       -- 页面名称
    url TEXT NOT NULL,                        -- 完整 URL
    description TEXT DEFAULT '',              -- 描述
    pinyin_full TEXT DEFAULT '',              -- 完整拼音 (如 "bai du")
    pinyin_initials TEXT DEFAULT '',          -- 拼音首字母 (如 "bd")
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 搜索历史（预留）
CREATE TABLE search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id INTEGER NOT NULL,
    last_searched_at TEXT NOT NULL DEFAULT (datetime('now')),
    search_count INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (page_id) REFERENCES page_index(id) ON DELETE CASCADE
);

-- 设置键值对
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

默认设置：
| Key | Default | 说明 |
|-----|---------|------|
| global_hotkey | Ctrl+K | 全局快捷键 |
| active_browser | chrome | 默认浏览器 |
| theme | system | 主题 (system/dark/light) |
| auto_analyze | false | 自动分析浏览记录 |
| db_version | 1 | 数据库版本 |

## 核心数据流

### 搜索 + 智能切换/打开

```
用户按下 Ctrl+K
  → Rust hotkey: Toggler 切换窗口可见性
  → 窗口弹出，输入框自动聚焦
用户输入关键词
  → 200ms 防抖（可配置）后 invoke("search_pages")
  → Rust: 查询 page_index 全表
  → nucleo 对 name/pinyin_full/pinyin_initials/url 加权评分
  → 同时获取 Chrome 已打开标签页
  → 对每个结果标记 is_open
  → 未索引但已打开的标签页也纳入结果
  → 按评分降序返回（已打开标签页 +100 boost）
用户 Enter 选结果
  → 先 invoke("hide_search_window") 关闭弹窗
  → 再 invoke("switch_or_open_tab")
  → Rust: 获取浏览器标签页列表
    → 已存在：AppleScript 切换到对应标签页
    → 不存在：open -a 新建标签页
    → Chrome 未运行：open -a 自动启动 Chrome
```

### 强制新建 (Ctrl+Enter / 中键点击)

```
跳过已打开检测 → 直接 invoke("open_new_tab") → open -a <url>
```

### 书签导入

```
invoke("import_bookmarks")
→ 读取 Chrome/Edge Bookmarks JSON
→ 扁平化嵌套书签树
→ 逐条 insert_page_ignore_dup（URL 标准化去重）
→ 返回导入数量
```

### 浏览记录分析

```
invoke("analyze_history") / 启动时自动执行
→ 遍历 Chrome/Edge/Arc/Brave History SQLite
→ 读取 visit_count ≥ 5 的 top 50 URL
→ 跨浏览器合并 + URL 标准化去重
→ 逐条 insert_page_ignore_dup
→ 返回导入数量
```

### 浏览器未运行处理

```
Chrome 关闭时：
  - get_open_tabs → 返回空列表
  - switch_or_open_tab → 调用 open -a "Google Chrome" <url>
    → 自动启动 Chrome 并打开目标页面
  - is_running 检测使用 pgrep（无需系统权限）
```

## 窗口管理

| 属性 | 值 | 说明 |
|------|-----|------|
| decorations | false | 无标题栏 |
| transparent | true | 透明窗口 |
| alwaysOnTop | true | 置顶 |
| skipTaskbar | true | 不显示在 Dock |
| visible | false | 初始隐藏 |
| width × height | 560 × 360 | 窗口尺寸 |
| macOSPrivateApi | true | 圆角 + 毛玻璃效果 |

关闭方式：
- Enter 选中后自动关闭
- ESC 键关闭
- 鼠标点击外部（失焦 200ms 后关闭）
- 快捷键 Ctrl+K 切换

## 浏览器通信 (macOS)

| 操作 | 方法 | 实现 |
|------|------|------|
| 检测运行状态 | `pgrep -q "Google Chrome"` | 零权限 |
| 获取标签页 | AppleScript 遍历窗口/标签 | URL + Title |
| 切换标签页 | AppleScript set active tab index | window_id + tab_index |
| 打开新标签页 | `open -a "Google Chrome" <url>` | 自动启动浏览器 |
| 书签读取 | 解析 `Application Support/.../Bookmarks` JSON | serde_json |
| 历史读取 | 解析 `Application Support/.../History` SQLite | rusqlite |

## 搜索评分策略

| 匹配字段 | 基础权重 |
|----------|----------|
| 直接子串匹配（名称） | 1000 - 位置偏移 |
| fuzzy 匹配（名称） | score + 500 |
| fuzzy 匹配（完整拼音） | score + 300 |
| fuzzy 匹配（拼音首字母） | score + 200 |
| fuzzy 匹配（URL） | score |
| 已打开标签页 | +100 boost |

## 主题系统

通过 CSS 类切换实现：

```css
/* 深色 */
.theme-dark {
  --bg-primary: rgba(30, 30, 32, 0.92);
  --text-primary: rgba(255, 255, 255, 0.92);
}

/* 浅色 - 白色毛玻璃 */
.theme-light {
  --bg-primary: rgba(255, 255, 255, 0.72);
  --text-primary: rgba(0, 0, 0, 0.88);
}
```

毛玻璃效果：`backdrop-filter: blur(30px) saturate(1.5)`

## 系统托盘

- 左键点击：切换搜索窗口
- 右键菜单：显示搜索框 / 管理页面索引 / 导入书签 / 设置 / 退出
- 图标使用裁剪后的 128×128 RGBA PNG

## URL 去重策略

标准化规则：
1. 转小写
2. 去除协议前缀 (https://, http://)
3. 去除 www. 前缀
4. 去除末尾斜线
5. 去除 hash 片段

示例：`https://www.github.com/` → `github.com`

## 构建与安装

```bash
# 开发
cargo tauri dev

# 生产构建
cargo tauri build

# 安装到 /Applications
cp -R src-tauri/target/release/bundle/macos/FlowTab.app /Applications/
```

## 产物

| 文件 | 大小 |
|------|------|
| flowtab 二进制 | ~14 MB |
| FlowTab.app | ~14 MB |
| FlowTab.dmg | ~5 MB |

## 关键依赖

### Rust (Cargo.toml)

```toml
tauri = { version = "2", features = ["tray-icon", "macos-private-api", "image-png"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-shell = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
nucleo = "0.5"
pinyin = "0.11"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Node (package.json)

```json
{
  "@tauri-apps/api": "^2.0.0",
  "@tauri-apps/plugin-global-shortcut": "^2.0.0",
  "@tauri-apps/plugin-shell": "^2.0.0",
  "svelte": "^5.0.0",
  "vite": "^6.0.0",
  "typescript": "^5.7.0"
}
```
