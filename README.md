# FlowTab

> 浏览器标签页管理工具 — 极速检索、智能去重、一键跳转

FlowTab 是一款基于 Rust + Tauri 的轻量级桌面应用。通过全局快捷键呼出搜索框，实时检索已保存的页面索引和浏览器中打开的标签页，智能判断切换或新建标签页，告别标签页堆积和重复打开的烦恼。

## 功能

- **全局快捷键召唤** — `Ctrl+K` 在任何界面呼出搜索浮窗，自动聚焦输入框
- **智能标签页路由** — 选中页面后自动检测浏览器是否已打开该 URL，已打开则切换，未打开则新建
- **已打开标签页展示** — 搜索结果中标记浏览器中已打开的页面，一目了然
- **多浏览器支持** — Chrome / Safari 完整控制，Firefox 仅支持新建标签页
- **强制新建模式** — `Ctrl+Enter` 跳过检测，始终新建标签页
- **拼音搜索** — 支持拼音全拼和首字母搜索中文页面名称
- **书签导入** — 一键导入 Chrome 浏览器书签
- **浏览记录分析** — 自动提取高频访问网站到搜索索引（可选）
- **URL 智能去重** — 标准化 URL 对比，避免重复条目
- **深色/浅色主题** — 白色毛玻璃 + 深色毛玻璃，跟随系统
- **系统托盘** — 菜单栏图标，右键快速访问设置和管理
- **浏览器冷启动** — Chrome / Safari 未运行时可自动启动并打开目标页面

## 安装

### 下载安装

从 [Releases](../../releases) 下载最新 `FlowTab.dmg`，拖入 `/Applications` 即可。

### 从源码构建

环境要求：
- Rust 1.70+
- Node.js 20+
- macOS 12.0+

```bash
git clone https://github.com/RichardsGR/FLowTab.git
cd FLowTab
npm install
cargo tauri build
# 产物位于 src-tauri/target/release/bundle/macos/FlowTab.app
```

## 使用

| 操作 | 快捷键 |
|------|--------|
| 呼出/隐藏搜索框 | `Ctrl+K` |
| 选中并打开/切换 | `Enter` |
| 强制新建标签页 | `Ctrl+Enter` |
| 上下选择 | `↑` `↓` |
| 关闭弹窗 | `Esc` 或点击外部 |

### 首次使用

1. 按 `Ctrl+K` 呼出搜索框
2. 点击托盘菜单 → **导入书签** 导入浏览器书签
3. 或点击托盘菜单 → **设置** → 开启**自动分析浏览记录** → **立即分析**
4. 开始搜索你的页面

## 技术栈

| 层 | 技术 |
|---|------|
| 桌面框架 | Tauri 2 |
| 前端 | Svelte 5 + TypeScript |
| 后端 | Rust |
| 数据库 | SQLite (rusqlite) |
| 搜索引擎 | nucleo + pinyin |
| 浏览器通信 | AppleScript / `open` 命令 |
| 构建工具 | Vite |

## 项目结构

```
FlowTab/
├── src-tauri/src/        # Rust 后端
│   ├── db/               # 数据库层
│   ├── search/           # 搜索引擎
│   ├── browsers/         # 浏览器控制
│   └── commands/         # IPC 命令
├── src/                   # Svelte 前端
│   ├── lib/               # 组件
│   ├── tauri/             # IPC 封装
│   └── styles/            # 样式
└── TECHNICAL.md           # 技术实现文档
```

## 平台支持

| 平台 | 状态 |
|------|------|
| macOS | 完整支持 |
| Windows | 规划中 |
| Linux | 规划中 |

## 许可

MIT License
