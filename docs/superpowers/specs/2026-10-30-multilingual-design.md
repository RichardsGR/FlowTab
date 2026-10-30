# FlowTab 多语言支持设计

- 日期：2026-10-30
- 分支：feat/glass-settings（叠加在当前未提交的玻璃设置页改动上）
- 状态：待评审

## 目标

1. 应用内 UI 支持 10 种常用语言。
2. 托盘菜单跟随语言设置同步本地化。
3. 修复现有 i18n 的响应式问题：切换语言后当前页面立即刷新。
4. 持久化语言设置，并迁移旧的 `zh` 值。

## 非目标

- RTL 布局（阿拉伯语 / 希伯来语）。
- 跟随系统语言自动选择。
- 翻译外包或母语审校流程。
- 浏览器扩展相关的多语言。

## 语言集合

| 代码 | 语言 |
|---|---|
| `en` | English |
| `zh-CN` | 简体中文 |
| `zh-TW` | 繁體中文 |
| `ja` | 日本語 |
| `ko` | 한국어 |
| `es` | Español |
| `fr` | Français |
| `de` | Deutsch |
| `pt` | Português |
| `ru` | Русский |

未知语言值一律回退到 `en`。

## 前端设计

### 类型与字典

```ts
export type Lang =
  | "en" | "zh-CN" | "zh-TW" | "ja" | "ko"
  | "es" | "fr" | "de" | "pt" | "ru";

const dict: Record<Lang, Record<string, string>> = { /* 10 份字典 */ };

export function i18n(
  lang: Lang,
  key: string,
  params?: Record<string, string | number>,
): string {
  let value = dict[lang]?.[key] ?? dict.en[key] ?? key;
  for (const [k, v] of Object.entries(params ?? {})) {
    value = value.replace(`{${k}}`, String(v));
  }
  return value;
}
```

- 删除模块级 `getLang` / `setLang`。
- 字典键与现有英文键完全一致，新增语言必须覆盖全部键。
- 英文是兜底语言。

### 状态与响应式

- `App.svelte` 持有 `let lang: Lang = $state("en")`。
- `onMount` 从设置读取语言；`onLangChange` 更新 `lang`。
- 所有组件已经接收 `lang` prop；把 `i18n("key")` 改为 `i18n(lang, "key")`。
- 模板表达式依赖 `lang` prop，Svelte 5 会在 prop 变化时重新求值，从而修复当前“切换语言不刷新”的问题。

涉及组件：

- `SearchPopup.svelte`
- `ResultItem.svelte`
- `PageIndexManager.svelte`
- `BookmarkImporter.svelte`
- `SettingsPanel.svelte`
- `ResultList.svelte`（仅透传 lang，无文案）

### 设置面板

- 语言下拉显示各语言本族名，值为语言代码。
- 读取设置时校验：值必须在 10 个代码中；旧值 `zh` 映射为 `zh-CN`；其他未知值回退 `en`。
- 保存后调用 `onLangChange(language)`，界面立即刷新。

## 后端托盘菜单

### 文案模块

新增 `src-tauri/src/tray_i18n.rs`：

```rust
pub struct TrayLabels {
    pub show_search: &'static str,
    pub manage_pages: &'static str,
    pub import_bookmarks: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
}

pub fn labels(lang: &str) -> TrayLabels { /* 10 种语言 + en 兜底 */ }
```

### 菜单构建与刷新

- 抽出 `pub(crate) fn build_tray_menu<R: Runtime>(app: &AppHandle<R>, lang: &str) -> Result<Menu<R>>`。
- 托盘使用固定 id `main`（`TrayIconBuilder::with_id("main")`）。
- 启动时从数据库读取 `language`，用对应语言构建菜单。
- `set_setting` 增加 `app: AppHandle` 参数；当 `key == "language"` 且写入成功后：
  - `app.tray_by_id("main")` 取到托盘；
  - 重新构建菜单并 `set_menu`。
- 菜单事件 id 保持不变。

## 持久化与迁移

- 设置键：`language`，默认 `en`。
- `db/init.rs` 增加：
  ```sql
  UPDATE settings SET value = 'zh-CN' WHERE key = 'language' AND value = 'zh';
  ```
- 前端读取时再做一次 `zh -> zh-CN` 兼容。

## 测试

- Rust 单测：
  - 10 种语言的托盘文案映射；
  - 未知语言回退英文。
- 构建验证：`cargo test`、`cargo check`、`npm run build`、`cargo tauri build --bundles app`。
- 手动矩阵：
  - 逐语言切换后，搜索框、设置、页面管理、导入页文案立即更新；
  - 托盘菜单同步更新；
  - 保存后重启，语言与托盘菜单保持；
  - 旧数据库 `zh` 值迁移为 `zh-CN`。

## 风险

- 机器翻译质量参差，后续可能需要母语审校。
- `set_setting` 增加 AppHandle 参数后需要确认 Tauri 注入与现有调用兼容。
- 托盘菜单重建期间旧菜单对象生命周期由 Tauri 管理，需在真实应用里验证。
- 当前玻璃设置页改动尚未提交，多语言改动会叠加在同一分支，提交时需分开描述。
