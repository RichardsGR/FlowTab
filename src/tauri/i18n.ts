export type Lang = "en" | "zh";

const dict: Record<Lang, Record<string, string>> = {
  en: {
    search_placeholder: "Search pages, URLs or pinyin...",
    searching: "Searching...",
    type_to_search: "Type to search...",
    no_results: "No results found",
    add_page: "Add Page",
    open: "Open",
    close_tab_title: "Close tab (Tab+⌫)",
    dup_tab_title: "Duplicate tab (Tab+⏎)",
    open_title: "Open (⏎)",
    settings: "Settings",
    back: "← Back",
    global_shortcut: "Global Shortcut",
    default_browser: "Default Browser",
    safari_first_use_hint: "First use requires allowing FlowTab to control Safari in System Settings → Privacy & Security → Automation.",
    automation_denied: "Permission needed: allow FlowTab to control {browser} in System Settings → Privacy & Security → Automation.",
    theme: "Theme",
    theme_system: "System",
    theme_dark: "Dark",
    theme_light: "Light",
    language: "Language",
    auto_analyze: "Auto-analyze browsing history",
    auto_analyze_desc: "Read browser history and auto-add frequently visited sites to the search index (visit count ≥ 5)",
    analyze_now: "Analyze Now",
    analyzing: "Analyzing...",
    save_settings: "Save Settings",
    saved: "Saved ✓",
    page_index: "Page Index",
    page_name_ph: "Page name (e.g. GitHub)",
    url_ph: "URL (e.g. https://github.com)",
    add: "Add",
    edit: "Edit",
    del: "Delete",
    save: "Save",
    cancel: "Cancel",
    clean_dupes: "Clean Duplicates",
    cleaning: "Cleaning...",
    import_bookmarks: "Import Bookmarks",
    import_desc: "Import bookmarks from Chrome. Duplicate URLs will be skipped.",
    start_import: "Start Import",
    importing: "Importing...",
    imported_n: "Imported {n} pages",
    import_success: "Successfully imported {n} bookmarks.",
    import_failed: "Import failed: {n}",
    removed_n: "Removed {n}",
    show_search: "Show Search",
    manage_pages: "Manage Pages",
    tray_import: "Import Bookmarks",
    tray_settings: "Settings",
    quit: "Quit",
  },
  zh: {
    search_placeholder: "搜索页面、URL 或拼音...",
    searching: "搜索中...",
    type_to_search: "输入关键词搜索...",
    no_results: "没有匹配的结果",
    add_page: "添加新页面",
    open: "已打开",
    close_tab_title: "关闭标签页 (Tab+⌫)",
    dup_tab_title: "新建相同标签页 (Tab+⏎)",
    open_title: "打开 (⏎)",
    settings: "设置",
    back: "← 返回",
    global_shortcut: "全局快捷键",
    default_browser: "默认浏览器",
    safari_first_use_hint: "首次使用需在 系统设置 → 隐私与安全性 → 自动化 中允许 FlowTab 控制 Safari。",
    automation_denied: "需要授权：请在 系统设置 → 隐私与安全性 → 自动化 中允许 FlowTab 控制 {browser}。",
    theme: "主题",
    theme_system: "跟随系统",
    theme_dark: "深色",
    theme_light: "浅色",
    language: "语言",
    auto_analyze: "自动分析浏览记录",
    auto_analyze_desc: "读取浏览器历史记录，将高频访问的网站自动添加到搜索索引（访问次数 ≥ 5）",
    analyze_now: "立即分析浏览记录",
    analyzing: "分析中...",
    save_settings: "保存设置",
    saved: "已保存 ✓",
    page_index: "页面索引管理",
    page_name_ph: "页面名称 (如: GitHub)",
    url_ph: "URL (如: https://github.com)",
    add: "添加",
    edit: "编辑",
    del: "删除",
    save: "保存",
    cancel: "取消",
    clean_dupes: "清理重复",
    cleaning: "清理中...",
    import_bookmarks: "导入浏览器书签",
    import_desc: "从 Chrome 浏览器的书签中导入页面。重复的 URL 将被自动跳过。",
    start_import: "开始导入",
    importing: "导入中...",
    imported_n: "已导入 {n} 个高频页面",
    import_success: "成功导入 {n} 个书签。",
    import_failed: "导入失败: {n}",
    removed_n: "已清理 {n} 条",
    show_search: "显示搜索框",
    manage_pages: "管理页面索引",
    tray_import: "导入书签",
    tray_settings: "设置",
    quit: "退出",
  },
};

let lang: Lang = "en";

export function i18n(key: string, params?: Record<string, string | number>): string {
  let val = dict[lang]?.[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      val = val.replace(`{${k}}`, String(v));
    }
  }
  return val;
}

export function getLang(): Lang {
  return lang;
}

export function setLang(l: Lang) {
  lang = l;
}
