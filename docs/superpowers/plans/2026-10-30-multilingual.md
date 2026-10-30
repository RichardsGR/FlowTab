# FlowTab 多语言支持 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 FlowTab 增加 10 种语言的 UI 与托盘菜单支持，并修复切换语言后文案不刷新的问题。

**Architecture:** 前端使用显式 `lang` 参数的 `i18n(lang, key, params)`，每种语言一份完整字典，TypeScript 在编译期强制字典完整；`App.svelte` 持有语言状态并向下传递。Rust 侧新增 `tray_i18n` 模块，托盘菜单使用固定 id，语言保存后通过 `AppHandle` 重建菜单。

**Tech Stack:** Svelte 5 / TypeScript / Tauri 2 / Rust

## Global Constraints

- 支持语言代码：`en`、`zh-CN`、`zh-TW`、`ja`、`ko`、`es`、`fr`、`de`、`pt`、`ru`。
- 未知语言回退 `en`；旧值 `zh` 映射为 `zh-CN`。
- 所有语言字典必须覆盖全部键；缺失由 TypeScript 报错。
- 托盘菜单事件 id 保持不变：`show_search`、`manage_pages`、`import`、`settings`、`quit`。
- 不引入新依赖。
- 工作区已有未提交的玻璃设置页改动（分支 `feat/glass-settings`），不要回退或覆盖；本计划叠加在其上。
- **提交策略：未获用户明确指示前不要执行 `git commit` / `git push`。**
- 每个任务结束运行任务内写明的验证命令。

---

### Task 1: i18n 核心、10 种语言字典与所有调用点

**Files:**
- Modify: `src/tauri/i18n.ts`
- Modify: `src/lib/App.svelte`
- Modify: `src/lib/SearchPopup.svelte`
- Modify: `src/lib/ResultItem.svelte`
- Modify: `src/lib/PageIndexManager.svelte`
- Modify: `src/lib/BookmarkImporter.svelte`
- Modify: `src/lib/SettingsPanel.svelte`

**Interfaces:**
- Produces:
  - `type Lang = "en" | "zh-CN" | "zh-TW" | "ja" | "ko" | "es" | "fr" | "de" | "pt" | "ru"`
  - `type TranslationKey`
  - `const LANGUAGES: { code: Lang; label: string }[]`
  - `function i18n(lang: Lang, key: TranslationKey, params?: Record<string, string | number>): string`
  - `function normalizeLang(value: string | null | undefined): Lang`
  - `function isLang(value: unknown): value is Lang`
- Consumes: 现有组件的 `lang` prop。

- [ ] **Step 1: 用完整内容替换 `src/tauri/i18n.ts`**

```ts
export type Lang =
  | "en"
  | "zh-CN"
  | "zh-TW"
  | "ja"
  | "ko"
  | "es"
  | "fr"
  | "de"
  | "pt"
  | "ru";

export const LANGUAGES: { code: Lang; label: string }[] = [
  { code: "en", label: "English" },
  { code: "zh-CN", label: "简体中文" },
  { code: "zh-TW", label: "繁體中文" },
  { code: "ja", label: "日本語" },
  { code: "ko", label: "한국어" },
  { code: "es", label: "Español" },
  { code: "fr", label: "Français" },
  { code: "de", label: "Deutsch" },
  { code: "pt", label: "Português" },
  { code: "ru", label: "Русский" },
];

const en = {
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
  safari_first_use_hint:
    "First use requires allowing FlowTab to control Safari in System Settings → Privacy & Security → Automation.",
  automation_denied:
    "Permission needed: allow FlowTab to control {browser} in System Settings → Privacy & Security → Automation.",
  theme: "Theme",
  theme_system: "System",
  theme_dark: "Dark",
  theme_light: "Light",
  language: "Language",
  auto_analyze: "Auto-analyze browsing history",
  auto_analyze_desc:
    "Read browser history and auto-add frequently visited sites to the search index (visit count ≥ 5)",
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
  import_desc:
    "Import bookmarks from Chrome. Duplicate URLs will be skipped.",
  start_import: "Start Import",
  importing: "Importing...",
  imported_n: "Imported {n} pages",
  import_success: "Successfully imported {n} bookmarks.",
  import_failed: "Import failed: {e}",
  removed_n: "Removed {n}",
  show_search: "Show Search",
  manage_pages: "Manage Pages",
  tray_import: "Import Bookmarks",
  tray_settings: "Settings",
  quit: "Quit",
};

export type TranslationKey = keyof typeof en;

const zhCN: Record<TranslationKey, string> = {
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
  safari_first_use_hint:
    "首次使用需在 系统设置 → 隐私与安全性 → 自动化 中允许 FlowTab 控制 Safari。",
  automation_denied:
    "需要授权：请在 系统设置 → 隐私与安全性 → 自动化 中允许 FlowTab 控制 {browser}。",
  theme: "主题",
  theme_system: "跟随系统",
  theme_dark: "深色",
  theme_light: "浅色",
  language: "语言",
  auto_analyze: "自动分析浏览记录",
  auto_analyze_desc:
    "读取浏览器历史记录，将高频访问的网站自动添加到搜索索引（访问次数 ≥ 5）",
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
  import_desc:
    "从 Chrome 浏览器的书签中导入页面。重复的 URL 将被自动跳过。",
  start_import: "开始导入",
  importing: "导入中...",
  imported_n: "已导入 {n} 个高频页面",
  import_success: "成功导入 {n} 个书签。",
  import_failed: "导入失败: {e}",
  removed_n: "已清理 {n} 条",
  show_search: "显示搜索框",
  manage_pages: "管理页面索引",
  tray_import: "导入书签",
  tray_settings: "设置",
  quit: "退出",
};

const zhTW: Record<TranslationKey, string> = {
  search_placeholder: "搜尋頁面、URL 或拼音...",
  searching: "搜尋中...",
  type_to_search: "輸入關鍵字搜尋...",
  no_results: "沒有符合的結果",
  add_page: "新增頁面",
  open: "已開啟",
  close_tab_title: "關閉分頁 (Tab+⌫)",
  dup_tab_title: "新增相同分頁 (Tab+⏎)",
  open_title: "開啟 (⏎)",
  settings: "設定",
  back: "← 返回",
  global_shortcut: "全域快速鍵",
  default_browser: "預設瀏覽器",
  safari_first_use_hint:
    "首次使用需在 系統設定 → 隱私權與安全性 → 自動化 中允許 FlowTab 控制 Safari。",
  automation_denied:
    "需要授權：請在 系統設定 → 隱私權與安全性 → 自動化 中允許 FlowTab 控制 {browser}。",
  theme: "主題",
  theme_system: "跟隨系統",
  theme_dark: "深色",
  theme_light: "淺色",
  language: "語言",
  auto_analyze: "自動分析瀏覽記錄",
  auto_analyze_desc:
    "讀取瀏覽器歷史記錄，將高頻造訪的網站自動加入搜尋索引（造訪次數 ≥ 5）",
  analyze_now: "立即分析瀏覽記錄",
  analyzing: "分析中...",
  save_settings: "儲存設定",
  saved: "已儲存 ✓",
  page_index: "頁面索引管理",
  page_name_ph: "頁面名稱 (例如: GitHub)",
  url_ph: "URL (例如: https://github.com)",
  add: "新增",
  edit: "編輯",
  del: "刪除",
  save: "儲存",
  cancel: "取消",
  clean_dupes: "清理重複",
  cleaning: "清理中...",
  import_bookmarks: "匯入瀏覽器書籤",
  import_desc:
    "從 Chrome 瀏覽器的書籤匯入頁面。重複的 URL 將自動略過。",
  start_import: "開始匯入",
  importing: "匯入中...",
  imported_n: "已匯入 {n} 個高頻頁面",
  import_success: "成功匯入 {n} 個書籤。",
  import_failed: "匯入失敗: {e}",
  removed_n: "已清理 {n} 筆",
  show_search: "顯示搜尋框",
  manage_pages: "管理頁面索引",
  tray_import: "匯入書籤",
  tray_settings: "設定",
  quit: "結束",
};

const ja: Record<TranslationKey, string> = {
  search_placeholder: "ページ、URL、ピンインを検索...",
  searching: "検索中...",
  type_to_search: "入力して検索...",
  no_results: "一致する結果がありません",
  add_page: "ページを追加",
  open: "開いています",
  close_tab_title: "タブを閉じる (Tab+⌫)",
  dup_tab_title: "同じタブを複製 (Tab+⏎)",
  open_title: "開く (⏎)",
  settings: "設定",
  back: "← 戻る",
  global_shortcut: "グローバルショートカット",
  default_browser: "既定のブラウザ",
  safari_first_use_hint:
    "初回使用時に、システム設定 → プライバシーとセキュリティ → オートメーション で FlowTab に Safari の制御を許可してください。",
  automation_denied:
    "権限が必要です：システム設定 → プライバシーとセキュリティ → オートメーション で FlowTab に {browser} の制御を許可してください。",
  theme: "テーマ",
  theme_system: "システム",
  theme_dark: "ダーク",
  theme_light: "ライト",
  language: "言語",
  auto_analyze: "閲覧履歴を自動分析",
  auto_analyze_desc:
    "ブラウザの履歴を読み取り、よく訪れるサイトを検索インデックスに自動追加します（訪問回数 ≥ 5）",
  analyze_now: "今すぐ分析",
  analyzing: "分析中...",
  save_settings: "設定を保存",
  saved: "保存済み ✓",
  page_index: "ページインデックス",
  page_name_ph: "ページ名 (例: GitHub)",
  url_ph: "URL (例: https://github.com)",
  add: "追加",
  edit: "編集",
  del: "削除",
  save: "保存",
  cancel: "キャンセル",
  clean_dupes: "重複を整理",
  cleaning: "整理中...",
  import_bookmarks: "ブックマークを読み込む",
  import_desc:
    "Chrome のブックマークからページを読み込みます。重複する URL は自動的にスキップされます。",
  start_import: "読み込みを開始",
  importing: "読み込み中...",
  imported_n: "{n} 件のページを読み込みました",
  import_success: "{n} 件のブックマークを読み込みました。",
  import_failed: "読み込みに失敗しました: {e}",
  removed_n: "{n} 件削除しました",
  show_search: "検索を表示",
  manage_pages: "ページを管理",
  tray_import: "ブックマークを読み込む",
  tray_settings: "設定",
  quit: "終了",
};

const ko: Record<TranslationKey, string> = {
  search_placeholder: "페이지, URL 또는 병음 검색...",
  searching: "검색 중...",
  type_to_search: "입력하여 검색...",
  no_results: "일치하는 결과가 없습니다",
  add_page: "페이지 추가",
  open: "열림",
  close_tab_title: "탭 닫기 (Tab+⌫)",
  dup_tab_title: "같은 탭 복제 (Tab+⏎)",
  open_title: "열기 (⏎)",
  settings: "설정",
  back: "← 뒤로",
  global_shortcut: "전역 단축키",
  default_browser: "기본 브라우저",
  safari_first_use_hint:
    "처음 사용할 때 시스템 설정 → 개인정보 보호 및 보안 → 자동화에서 FlowTab이 Safari를 제어하도록 허용해야 합니다.",
  automation_denied:
    "권한이 필요합니다: 시스템 설정 → 개인정보 보호 및 보안 → 자동화에서 FlowTab이 {browser}를 제어하도록 허용하세요.",
  theme: "테마",
  theme_system: "시스템",
  theme_dark: "다크",
  theme_light: "라이트",
  language: "언어",
  auto_analyze: "방문 기록 자동 분석",
  auto_analyze_desc:
    "브라우저 방문 기록을 읽고 자주 방문하는 사이트를 검색 인덱스에 자동 추가합니다(방문 횟수 ≥ 5).",
  analyze_now: "지금 분석",
  analyzing: "분석 중...",
  save_settings: "설정 저장",
  saved: "저장됨 ✓",
  page_index: "페이지 인덱스",
  page_name_ph: "페이지 이름 (예: GitHub)",
  url_ph: "URL (예: https://github.com)",
  add: "추가",
  edit: "편집",
  del: "삭제",
  save: "저장",
  cancel: "취소",
  clean_dupes: "중복 정리",
  cleaning: "정리 중...",
  import_bookmarks: "북마크 가져오기",
  import_desc:
    "Chrome 북마크에서 페이지를 가져옵니다. 중복된 URL은 자동으로 건너뜁니다.",
  start_import: "가져오기 시작",
  importing: "가져오는 중...",
  imported_n: "{n}개 페이지를 가져왔습니다",
  import_success: "북마크 {n}개를 가져왔습니다.",
  import_failed: "가져오기 실패: {e}",
  removed_n: "{n}개 제거됨",
  show_search: "검색 표시",
  manage_pages: "페이지 관리",
  tray_import: "북마크 가져오기",
  tray_settings: "설정",
  quit: "종료",
};

const es: Record<TranslationKey, string> = {
  search_placeholder: "Busca páginas, URL o pinyin...",
  searching: "Buscando...",
  type_to_search: "Escribe para buscar...",
  no_results: "No se encontraron resultados",
  add_page: "Añadir página",
  open: "Abierta",
  close_tab_title: "Cerrar pestaña (Tab+⌫)",
  dup_tab_title: "Duplicar pestaña (Tab+⏎)",
  open_title: "Abrir (⏎)",
  settings: "Ajustes",
  back: "← Atrás",
  global_shortcut: "Atajo global",
  default_browser: "Navegador predeterminado",
  safari_first_use_hint:
    "El primer uso requiere permitir que FlowTab controle Safari en Ajustes del Sistema → Privacidad y seguridad → Automatización.",
  automation_denied:
    "Se necesita permiso: permite que FlowTab controle {browser} en Ajustes del Sistema → Privacidad y seguridad → Automatización.",
  theme: "Tema",
  theme_system: "Sistema",
  theme_dark: "Oscuro",
  theme_light: "Claro",
  language: "Idioma",
  auto_analyze: "Analizar automáticamente el historial",
  auto_analyze_desc:
    "Lee el historial del navegador y añade automáticamente los sitios visitados con frecuencia al índice de búsqueda (visitas ≥ 5).",
  analyze_now: "Analizar ahora",
  analyzing: "Analizando...",
  save_settings: "Guardar ajustes",
  saved: "Guardado ✓",
  page_index: "Índice de páginas",
  page_name_ph: "Nombre de la página (p. ej., GitHub)",
  url_ph: "URL (p. ej., https://github.com)",
  add: "Añadir",
  edit: "Editar",
  del: "Eliminar",
  save: "Guardar",
  cancel: "Cancelar",
  clean_dupes: "Limpiar duplicados",
  cleaning: "Limpiando...",
  import_bookmarks: "Importar marcadores",
  import_desc:
    "Importa marcadores desde Chrome. Las URL duplicadas se omitirán.",
  start_import: "Iniciar importación",
  importing: "Importando...",
  imported_n: "Se importaron {n} páginas",
  import_success: "Se importaron {n} marcadores.",
  import_failed: "Error al importar: {e}",
  removed_n: "{n} eliminados",
  show_search: "Mostrar búsqueda",
  manage_pages: "Gestionar páginas",
  tray_import: "Importar marcadores",
  tray_settings: "Ajustes",
  quit: "Salir",
};

const fr: Record<TranslationKey, string> = {
  search_placeholder: "Rechercher des pages, URL ou pinyin...",
  searching: "Recherche...",
  type_to_search: "Saisissez pour rechercher...",
  no_results: "Aucun résultat",
  add_page: "Ajouter une page",
  open: "Ouverte",
  close_tab_title: "Fermer l'onglet (Tab+⌫)",
  dup_tab_title: "Dupliquer l'onglet (Tab+⏎)",
  open_title: "Ouvrir (⏎)",
  settings: "Réglages",
  back: "← Retour",
  global_shortcut: "Raccourci global",
  default_browser: "Navigateur par défaut",
  safari_first_use_hint:
    "La première utilisation nécessite d'autoriser FlowTab à contrôler Safari dans Réglages Système → Confidentialité et sécurité → Automatisation.",
  automation_denied:
    "Autorisation requise : autorisez FlowTab à contrôler {browser} dans Réglages Système → Confidentialité et sécurité → Automatisation.",
  theme: "Thème",
  theme_system: "Système",
  theme_dark: "Sombre",
  theme_light: "Clair",
  language: "Langue",
  auto_analyze: "Analyser automatiquement l'historique",
  auto_analyze_desc:
    "Lit l'historique du navigateur et ajoute automatiquement les sites fréquemment visités à l'index de recherche (visites ≥ 5).",
  analyze_now: "Analyser maintenant",
  analyzing: "Analyse en cours...",
  save_settings: "Enregistrer les réglages",
  saved: "Enregistré ✓",
  page_index: "Index des pages",
  page_name_ph: "Nom de la page (ex. GitHub)",
  url_ph: "URL (ex. https://github.com)",
  add: "Ajouter",
  edit: "Modifier",
  del: "Supprimer",
  save: "Enregistrer",
  cancel: "Annuler",
  clean_dupes: "Nettoyer les doublons",
  cleaning: "Nettoyage...",
  import_bookmarks: "Importer les favoris",
  import_desc:
    "Importe les favoris depuis Chrome. Les URL en double seront ignorées.",
  start_import: "Lancer l'importation",
  importing: "Importation...",
  imported_n: "{n} pages importées",
  import_success: "{n} favoris importés.",
  import_failed: "Échec de l'importation : {e}",
  removed_n: "{n} supprimés",
  show_search: "Afficher la recherche",
  manage_pages: "Gérer les pages",
  tray_import: "Importer les favoris",
  tray_settings: "Réglages",
  quit: "Quitter",
};

const de: Record<TranslationKey, string> = {
  search_placeholder: "Seiten, URLs oder Pinyin durchsuchen...",
  searching: "Wird gesucht...",
  type_to_search: "Zum Suchen tippen...",
  no_results: "Keine Ergebnisse gefunden",
  add_page: "Seite hinzufügen",
  open: "Offen",
  close_tab_title: "Tab schließen (Tab+⌫)",
  dup_tab_title: "Tab duplizieren (Tab+⏎)",
  open_title: "Öffnen (⏎)",
  settings: "Einstellungen",
  back: "← Zurück",
  global_shortcut: "Globaler Kurzbefehl",
  default_browser: "Standardbrowser",
  safari_first_use_hint:
    "Bei der ersten Verwendung muss FlowTab in den Systemeinstellungen → Datenschutz & Sicherheit → Automatisierung die Steuerung von Safari erlauben.",
  automation_denied:
    "Berechtigung erforderlich: Erlauben Sie FlowTab in den Systemeinstellungen → Datenschutz & Sicherheit → Automatisierung die Steuerung von {browser}.",
  theme: "Erscheinungsbild",
  theme_system: "System",
  theme_dark: "Dunkel",
  theme_light: "Hell",
  language: "Sprache",
  auto_analyze:
    "Browserverlauf automatisch analysieren",
  auto_analyze_desc:
    "Liest den Browserverlauf und fügt häufig besuchte Seiten automatisch zum Suchindex hinzu (Aufrufe ≥ 5).",
  analyze_now: "Jetzt analysieren",
  analyzing: "Wird analysiert...",
  save_settings: "Einstellungen speichern",
  saved: "Gespeichert ✓",
  page_index: "Seitenindex",
  page_name_ph: "Seitenname (z. B. GitHub)",
  url_ph: "URL (z. B. https://github.com)",
  add: "Hinzufügen",
  edit: "Bearbeiten",
  del: "Löschen",
  save: "Speichern",
  cancel: "Abbrechen",
  clean_dupes: "Duplikate bereinigen",
  cleaning: "Wird bereinigt...",
  import_bookmarks: "Lesezeichen importieren",
  import_desc:
    "Importiert Lesezeichen aus Chrome. Doppelte URLs werden übersprungen.",
  start_import: "Import starten",
  importing: "Wird importiert...",
  imported_n: "{n} Seiten importiert",
  import_success: "{n} Lesezeichen importiert.",
  import_failed: "Import fehlgeschlagen: {e}",
  removed_n: "{n} entfernt",
  show_search: "Suche anzeigen",
  manage_pages: "Seiten verwalten",
  tray_import: "Lesezeichen importieren",
  tray_settings: "Einstellungen",
  quit: "Beenden",
};

const pt: Record<TranslationKey, string> = {
  search_placeholder: "Pesquisar páginas, URLs ou pinyin...",
  searching: "Pesquisando...",
  type_to_search: "Digite para pesquisar...",
  no_results: "Nenhum resultado encontrado",
  add_page: "Adicionar página",
  open: "Aberta",
  close_tab_title: "Fechar aba (Tab+⌫)",
  dup_tab_title: "Duplicar aba (Tab+⏎)",
  open_title: "Abrir (⏎)",
  settings: "Configurações",
  back: "← Voltar",
  global_shortcut: "Atalho global",
  default_browser: "Navegador padrão",
  safari_first_use_hint:
    "O primeiro uso exige permitir que o FlowTab controle o Safari em Ajustes do Sistema → Privacidade e Segurança → Automação.",
  automation_denied:
    "Permissão necessária: permita que o FlowTab controle o {browser} em Ajustes do Sistema → Privacidade e Segurança → Automação.",
  theme: "Tema",
  theme_system: "Sistema",
  theme_dark: "Escuro",
  theme_light: "Claro",
  language: "Idioma",
  auto_analyze: "Analisar automaticamente o histórico",
  auto_analyze_desc:
    "Lê o histórico do navegador e adiciona automaticamente os sites visitados com frequência ao índice de pesquisa (visitas ≥ 5).",
  analyze_now: "Analisar agora",
  analyzing: "Analisando...",
  save_settings: "Salvar configurações",
  saved: "Salvo ✓",
  page_index: "Índice de páginas",
  page_name_ph: "Nome da página (ex.: GitHub)",
  url_ph: "URL (ex.: https://github.com)",
  add: "Adicionar",
  edit: "Editar",
  del: "Excluir",
  save: "Salvar",
  cancel: "Cancelar",
  clean_dupes: "Limpar duplicados",
  cleaning: "Limpando...",
  import_bookmarks: "Importar favoritos",
  import_desc:
    "Importa favoritos do Chrome. URLs duplicadas serão ignoradas.",
  start_import: "Iniciar importação",
  importing: "Importando...",
  imported_n: "{n} páginas importadas",
  import_success: "{n} favoritos importados.",
  import_failed: "Falha na importação: {e}",
  removed_n: "{n} removidos",
  show_search: "Mostrar pesquisa",
  manage_pages: "Gerenciar páginas",
  tray_import: "Importar favoritos",
  tray_settings: "Configurações",
  quit: "Sair",
};

const ru: Record<TranslationKey, string> = {
  search_placeholder: "Поиск страниц, URL или пиньиня...",
  searching: "Поиск...",
  type_to_search: "Введите запрос...",
  no_results: "Ничего не найдено",
  add_page: "Добавить страницу",
  open: "Открыто",
  close_tab_title: "Закрыть вкладку (Tab+⌫)",
  dup_tab_title: "Дублировать вкладку (Tab+⏎)",
  open_title: "Открыть (⏎)",
  settings: "Настройки",
  back: "← Назад",
  global_shortcut: "Глобальная горячая клавиша",
  default_browser: "Браузер по умолчанию",
  safari_first_use_hint:
    "При первом использовании разрешите FlowTab управлять Safari в «Системные настройки → Конфиденциальность и безопасность → Автоматизация».",
  automation_denied:
    "Требуется разрешение: разрешите FlowTab управлять {browser} в «Системные настройки → Конфиденциальность и безопасность → Автоматизация».",
  theme: "Тема",
  theme_system: "Системная",
  theme_dark: "Тёмная",
  theme_light: "Светлая",
  language: "Язык",
  auto_analyze: "Автоматически анализировать историю",
  auto_analyze_desc:
    "Читает историю браузера и автоматически добавляет часто посещаемые сайты в поисковый индекс (посещений ≥ 5).",
  analyze_now: "Анализировать сейчас",
  analyzing: "Анализ...",
  save_settings: "Сохранить настройки",
  saved: "Сохранено ✓",
  page_index: "Индекс страниц",
  page_name_ph: "Название страницы (например, GitHub)",
  url_ph: "URL (например, https://github.com)",
  add: "Добавить",
  edit: "Изменить",
  del: "Удалить",
  save: "Сохранить",
  cancel: "Отмена",
  clean_dupes: "Удалить дубликаты",
  cleaning: "Очистка...",
  import_bookmarks: "Импортировать закладки",
  import_desc:
    "Импортирует закладки из Chrome. Повторяющиеся URL пропускаются.",
  start_import: "Начать импорт",
  importing: "Импорт...",
  imported_n: "Импортировано страниц: {n}",
  import_success: "Импортировано закладок: {n}.",
  import_failed: "Ошибка импорта: {e}",
  removed_n: "Удалено: {n}",
  show_search: "Показать поиск",
  manage_pages: "Управление страницами",
  tray_import: "Импортировать закладки",
  tray_settings: "Настройки",
  quit: "Выход",
};

const dict: Record<Lang, Record<TranslationKey, string>> = {
  en,
  "zh-CN": zhCN,
  "zh-TW": zhTW,
  ja,
  ko,
  es,
  fr,
  de,
  pt,
  ru,
};

export function isLang(value: unknown): value is Lang {
  return typeof value === "string" && value in dict;
}

export function normalizeLang(value: string | null | undefined): Lang {
  if (value === "zh") return "zh-CN";
  return isLang(value) ? value : "en";
}

export function i18n(
  lang: Lang,
  key: TranslationKey,
  params?: Record<string, string | number>,
): string {
  let value = dict[lang]?.[key] ?? dict.en[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      value = value.replace(`{${k}}`, String(v));
    }
  }
  return value;
}
```

- [ ] **Step 2: 更新 `App.svelte`**

把 import 改为：

```ts
  import { normalizeLang, type Lang } from "../tauri/i18n";
```

状态改为：

```ts
  let lang: Lang = $state("en");
```

`onLangChange` 改为：

```ts
  function onLangChange(l: Lang) {
    lang = l;
  }
```

`onMount` 中把语言读取改为：

```ts
      lang = normalizeLang(await getSetting("language"));
```

删除原来对 `savedLang` 的判断和 `setLang` 调用。

- [ ] **Step 3: 更新 `SearchPopup.svelte` 的调用点**

```ts
      automationError = i18n(lang, "automation_denied", { browser });
```

```svelte
        placeholder={i18n(lang, "search_placeholder")}
```

```svelte
      <div class="empty-state">{i18n(lang, "searching")}</div>
```

```svelte
      <EmptyState message={i18n(lang, "type_to_search")} icon="🔍" />
```

```svelte
      <EmptyState message={i18n(lang, "no_results")} icon="📭">
        <span class="empty-state-action" onclick={() => onNavigate("manager")}>{i18n(lang, "add_page")}</span>
      </EmptyState>
```

- [ ] **Step 4: 更新 `ResultItem.svelte` 的调用点**

```svelte
    <span class="result-item-badge badge-open">{i18n(lang, "open")}</span>
    {#if selected}
      <button class="result-item-action" title={i18n(lang, "close_tab_title")}
        onclick={(e) => { e.stopPropagation(); onClose?.(item); }}>
        ✕
      </button>
      <button class="result-item-action primary" title={i18n(lang, "dup_tab_title")}
        onclick={(e) => { e.stopPropagation(); onForceNew?.(item); }}>
        +
      </button>
    {/if}
  {:else if selected}
    <span class="result-item-action primary" title={i18n(lang, "open_title")}>
      ⏎
    </span>
  {/if}
```

- [ ] **Step 5: 更新 `PageIndexManager.svelte` 的调用点**

```svelte
    <button class="btn" onclick={onBack}>{i18n(lang, "back")}</button>
    <h2 style="margin-bottom:0;">{i18n(lang, "page_index")}</h2>
    <div style="flex:1"></div>
    <button class="btn btn-danger" onclick={handleClean} disabled={cleaning}>
      {cleaning ? i18n(lang, "cleaning") : i18n(lang, "clean_dupes")}
    </button>
    {#if cleanedCount !== null}
      <span style="font-size:13px;color:var(--text-secondary);">{i18n(lang, "removed_n", { n: cleanedCount })}</span>
    {/if}
```

```svelte
      <input type="text" placeholder={i18n(lang, "page_name_ph")} bind:value={newName} />
      <input type="text" placeholder={i18n(lang, "url_ph")} bind:value={newUrl} />
      <button class="btn btn-primary" onclick={handleAdd}>{i18n(lang, "add")}</button>
```

```svelte
          <button class="btn btn-primary" onclick={() => handleUpdate(page.id)}>{i18n(lang, "save")}</button>
          <button class="btn" onclick={() => (editingId = 0)}>{i18n(lang, "cancel")}</button>
```

```svelte
            <button class="btn" onclick={() => startEdit(page)}>{i18n(lang, "edit")}</button>
            <button class="btn btn-danger" onclick={() => handleDelete(page.id)}>{i18n(lang, "del")}</button>
```

- [ ] **Step 6: 更新 `BookmarkImporter.svelte` 的调用点**

```svelte
    <button class="btn" onclick={onBack}>{i18n(lang, "back")}</button>
    <h2 style="margin-bottom:0;">{i18n(lang, "import_bookmarks")}</h2>
```

```svelte
    {i18n(lang, "import_desc")}
```

```svelte
    {importing ? i18n(lang, "importing") : i18n(lang, "start_import")}
```

```svelte
      {i18n(lang, "import_success", { n: importedCount })}
```

```svelte
      {i18n(lang, "import_failed", { e: error })}
```

- [ ] **Step 7: 更新 `SettingsPanel.svelte` 的调用点与语言下拉**

import 改为：

```ts
  import { i18n, LANGUAGES, normalizeLang, type Lang } from "../tauri/i18n";
```

`loadSettings` 中语言读取改为：

```ts
      language = normalizeLang(await getSetting("language"));
```

语言下拉改为：

```svelte
    <select bind:value={language}>
      {#each LANGUAGES as l (l.code)}
        <option value={l.code}>{l.label}</option>
      {/each}
    </select>
```

其余调用点统一加 `lang` 参数：

```svelte
    <button class="btn" onclick={onBack}>{i18n(lang, "back")}</button>
    <h2 style="margin-bottom:0;">{i18n(lang, "settings")}</h2>
```

```svelte
    <label>{i18n(lang, "global_shortcut")}</label>
```

```svelte
    <label>{i18n(lang, "default_browser")}</label>
```

```svelte
        {i18n(lang, "safari_first_use_hint")}
```

```svelte
    <label>{i18n(lang, "theme")}</label>
    <select bind:value={theme}>
      <option value="system">{i18n(lang, "theme_system")}</option>
      <option value="dark">{i18n(lang, "theme_dark")}</option>
      <option value="light">{i18n(lang, "theme_light")}</option>
    </select>
```

```svelte
    <label>{i18n(lang, "language")}</label>
```

```svelte
      {i18n(lang, "auto_analyze")}
```

```svelte
      {i18n(lang, "auto_analyze_desc")}
```

```svelte
      {analyzing ? i18n(lang, "analyzing") : i18n(lang, "analyze_now")}
```

```svelte
        {i18n(lang, "imported_n", { n: analyzedCount })}
```

```svelte
      {saved ? i18n(lang, "saved") : i18n(lang, "save_settings")}
```

- [ ] **Step 8: 验证**

```bash
npm run build
```

预期：TypeScript/Svelte 构建通过。若某语言字典缺键，`Record<TranslationKey, string>` 会报错。

---

### Task 2: 数据库语言迁移

**Files:**
- Modify: `src-tauri/src/db/init.rs`

**Interfaces:**
- Consumes: `settings` 表。
- Produces: 启动时把 `language = 'zh'` 迁移为 `'zh-CN'`。

- [ ] **Step 1: 在 `initialize_database` 的 `execute_batch` 中追加迁移 SQL**

在现有 `active_browser = 'edge'` 迁移之后加入：

```sql
        UPDATE settings SET value = 'zh-CN' WHERE key = 'language' AND value = 'zh';
```

- [ ] **Step 2: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

---

### Task 3: Rust 托盘菜单本地化

**Files:**
- Create: `src-tauri/src/tray_i18n.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/settings_commands.rs`

**Interfaces:**
- Produces:
  - `tray_i18n::TrayLabels`（5 个 `&'static str` 字段）
  - `tray_i18n::labels(lang: &str) -> TrayLabels`
  - `pub(crate) fn build_tray_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>, lang: &str) -> tauri::Result<tauri::menu::Menu<R>>`
- Consumes: `BrowserType` 无关；仅 `db::queries::get_setting` 与 Tauri menu/tray API。

- [ ] **Step 1: 新建 `src-tauri/src/tray_i18n.rs`**

```rust
pub struct TrayLabels {
    pub show_search: &'static str,
    pub manage_pages: &'static str,
    pub import_bookmarks: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
}

pub fn labels(lang: &str) -> TrayLabels {
    match lang {
        "zh-CN" => TrayLabels {
            show_search: "显示搜索框",
            manage_pages: "管理页面索引",
            import_bookmarks: "导入书签",
            settings: "设置",
            quit: "退出",
        },
        "zh-TW" => TrayLabels {
            show_search: "顯示搜尋框",
            manage_pages: "管理頁面索引",
            import_bookmarks: "匯入書籤",
            settings: "設定",
            quit: "結束",
        },
        "ja" => TrayLabels {
            show_search: "検索を表示",
            manage_pages: "ページを管理",
            import_bookmarks: "ブックマークを読み込む",
            settings: "設定",
            quit: "終了",
        },
        "ko" => TrayLabels {
            show_search: "검색 표시",
            manage_pages: "페이지 관리",
            import_bookmarks: "북마크 가져오기",
            settings: "설정",
            quit: "종료",
        },
        "es" => TrayLabels {
            show_search: "Mostrar búsqueda",
            manage_pages: "Gestionar páginas",
            import_bookmarks: "Importar marcadores",
            settings: "Ajustes",
            quit: "Salir",
        },
        "fr" => TrayLabels {
            show_search: "Afficher la recherche",
            manage_pages: "Gérer les pages",
            import_bookmarks: "Importer les favoris",
            settings: "Réglages",
            quit: "Quitter",
        },
        "de" => TrayLabels {
            show_search: "Suche anzeigen",
            manage_pages: "Seiten verwalten",
            import_bookmarks: "Lesezeichen importieren",
            settings: "Einstellungen",
            quit: "Beenden",
        },
        "pt" => TrayLabels {
            show_search: "Mostrar pesquisa",
            manage_pages: "Gerenciar páginas",
            import_bookmarks: "Importar favoritos",
            settings: "Configurações",
            quit: "Sair",
        },
        "ru" => TrayLabels {
            show_search: "Показать поиск",
            manage_pages: "Управление страницами",
            import_bookmarks: "Импортировать закладки",
            settings: "Настройки",
            quit: "Выход",
        },
        _ => TrayLabels {
            show_search: "Show Search",
            manage_pages: "Manage Pages",
            import_bookmarks: "Import Bookmarks",
            settings: "Settings",
            quit: "Quit",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_languages() {
        assert_eq!(labels("zh-CN").settings, "设置");
        assert_eq!(labels("zh-TW").settings, "設定");
        assert_eq!(labels("ja").quit, "終了");
        assert_eq!(labels("ko").settings, "설정");
        assert_eq!(labels("es").quit, "Salir");
        assert_eq!(labels("fr").settings, "Réglages");
        assert_eq!(labels("de").quit, "Beenden");
        assert_eq!(labels("pt").settings, "Configurações");
        assert_eq!(labels("ru").quit, "Выход");
    }

    #[test]
    fn falls_back_to_english() {
        let labels = labels("unknown");
        assert_eq!(labels.show_search, "Show Search");
        assert_eq!(labels.manage_pages, "Manage Pages");
        assert_eq!(labels.import_bookmarks, "Import Bookmarks");
        assert_eq!(labels.settings, "Settings");
        assert_eq!(labels.quit, "Quit");
    }
}
```

- [ ] **Step 2: 在 `lib.rs` 注册模块并抽取菜单构建函数**

顶部加入模块声明，并把现有的 `use tauri::menu::{MenuBuilder, MenuItemBuilder};` 替换为包含 `Menu` 的版本：

```rust
mod tray_i18n;
use crate::db::queries;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};
use tauri::Runtime;
```

把现有的内联菜单构建替换为：

```rust
            let lang = {
                let state = app.state::<Mutex<AppState>>();
                match state.lock() {
                    Ok(guard) => queries::get_setting(&guard.db, "language")
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "en".into()),
                    Err(_) => "en".into(),
                }
            };

            let menu = build_tray_menu(&handle, &lang)?;
```

在 `run()` 之外加入：

```rust
pub(crate) fn build_tray_menu<R: Runtime>(
    app: &tauri::AppHandle<R>,
    lang: &str,
) -> tauri::Result<Menu<R>> {
    let labels = tray_i18n::labels(lang);

    let show_search =
        MenuItemBuilder::with_id("show_search", labels.show_search).build(app)?;
    let manage_pages =
        MenuItemBuilder::with_id("manage_pages", labels.manage_pages).build(app)?;
    let import =
        MenuItemBuilder::with_id("import", labels.import_bookmarks).build(app)?;
    let settings =
        MenuItemBuilder::with_id("settings", labels.settings).build(app)?;
    let quit = MenuItemBuilder::with_id("quit", labels.quit).build(app)?;

    MenuBuilder::new(app)
        .item(&show_search)
        .item(&manage_pages)
        .item(&import)
        .item(&settings)
        .separator()
        .item(&quit)
        .build()
}
```

托盘构建改为固定 id：

```rust
            let _tray = TrayIconBuilder::with_id("main")
                .icon(tray_icon)
```

- [ ] **Step 3: 让 `set_setting` 在语言变化时重建菜单**

`src-tauri/src/commands/settings_commands.rs` 的 `set_setting` 改为：

```rust
#[tauri::command]
pub fn set_setting(
    app: tauri::AppHandle,
    key: String,
    value: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        queries::set_setting(&app_state.db, &key, &value).map_err(|e| e.to_string())?;
    }

    if key == "language" {
        if let Some(tray) = app.tray_by_id("main") {
            let menu = crate::build_tray_menu(&app, &value).map_err(|e| e.to_string())?;
            tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
```

- [ ] **Step 4: 验证**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

预期：`tray_i18n` 两个测试通过；编译无错误。

---

### Task 4: 全量验证与安装

**Files:** 无新增。

- [ ] **Step 1: 构建与测试**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
cargo tauri build --bundles app
```

- [ ] **Step 2: 安装并手动验证**

```bash
pkill -x flowtab 2>/dev/null
ditto src-tauri/target/release/bundle/macos/FlowTab.app /Applications/FlowTab.app
open /Applications/FlowTab.app
```

手动检查：

- [ ] 设置 → 语言下拉显示 10 种语言的本族名。
- [ ] 依次切换简体中文、繁體中文、日本語、한국어、Español、Français、Deutsch、Português、Русский，当前设置页文案立即变化。
- [ ] 打开搜索框，占位符与空状态文案随语言变化。
- [ ] 打开页面管理、导入页，文案随语言变化。
- [ ] 每次保存语言后，托盘菜单 5 个菜单项立即变化。
- [ ] 重启应用后，语言与托盘菜单保持。
- [ ] 旧数据库 `language = 'zh'` 自动变成 `zh-CN`（可用 sqlite3 检查）。

- [ ] **Step 3: 汇总改动并等待用户决定是否提交**

```bash
git status -sb
git diff --stat
```

不要自动提交；报告结果并询问是否提交。
