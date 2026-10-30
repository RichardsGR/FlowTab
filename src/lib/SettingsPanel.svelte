<script lang="ts">
  import { onMount } from "svelte";
  import { getSetting, setSetting, analyzeHistory } from "../tauri/commands";
  import { i18n, type Lang } from "../tauri/i18n";

  let {
    lang,
    onBack,
    onLangChange,
  }: {
    lang: Lang;
    onBack: () => void;
    onLangChange: (l: Lang) => void;
  } = $props();

  let hotkey = $state("Ctrl+K");
  let activeBrowser = $state("chrome");
  let theme = $state("system");
  let language = $state<Lang>(lang);
  let autoAnalyze = $state(false);
  let saved = $state(false);
  let analyzing = $state(false);
  let analyzedCount = $state<number | null>(null);

  function applyTheme(t: string) {
    const html = document.documentElement;
    html.classList.remove("theme-dark", "theme-light");
    let resolved = t;
    if (t === "system") {
      resolved = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
    html.classList.add(resolved === "light" ? "theme-light" : "theme-dark");
  }

  async function loadSettings() {
    try {
      hotkey = (await getSetting("global_hotkey")) ?? "Ctrl+K";
      const savedBrowser = (await getSetting("active_browser")) ?? "chrome";
      activeBrowser = ["chrome", "safari", "firefox"].includes(savedBrowser)
        ? savedBrowser
        : "chrome";
      theme = (await getSetting("theme")) ?? "system";
      const savedLang = await getSetting("language");
      language = (savedLang === "zh" || savedLang === "en") ? savedLang as Lang : "en";
      autoAnalyze = (await getSetting("auto_analyze")) === "true";
      applyTheme(theme);
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }

  async function handleSave() {
    try {
      await setSetting("global_hotkey", hotkey);
      await setSetting("active_browser", activeBrowser);
      await setSetting("theme", theme);
      await setSetting("language", language);
      await setSetting("auto_analyze", autoAnalyze ? "true" : "false");
      applyTheme(theme);
      onLangChange(language);
      saved = true;
      setTimeout(() => (saved = false), 2000);
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  async function handleAnalyze() {
    analyzing = true;
    analyzedCount = null;
    try {
      analyzedCount = await analyzeHistory();
    } catch (e) {
      console.error("Analysis failed:", e);
    } finally {
      analyzing = false;
    }
  }

  onMount(() => {
    loadSettings();
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      if (theme === "system") applyTheme("system");
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  });
</script>

<div class="page-manager">
  <div style="display:flex;align-items:center;gap:12px;margin-bottom:16px;">
    <button class="btn" onclick={onBack}>{i18n("back")}</button>
    <h2 style="margin-bottom:0;">{i18n("settings")}</h2>
  </div>

  <div class="settings-group">
    <label>{i18n("global_shortcut")}</label>
    <input type="text" bind:value={hotkey} placeholder="e.g. Ctrl+K" />
  </div>

  <div class="settings-group">
    <label>{i18n("default_browser")}</label>
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
  </div>

  <div class="settings-group">
    <label>{i18n("theme")}</label>
    <select bind:value={theme}>
      <option value="system">{i18n("theme_system")}</option>
      <option value="dark">{i18n("theme_dark")}</option>
      <option value="light">{i18n("theme_light")}</option>
    </select>
  </div>

  <div class="settings-group">
    <label>{i18n("language")}</label>
    <select bind:value={language}>
      <option value="zh">中文</option>
      <option value="en">English</option>
    </select>
  </div>

  <div class="settings-group" style="border-top:1px solid var(--border);padding-top:16px;margin-top:16px;">
    <label style="display:flex;align-items:center;gap:8px;">
      <input type="checkbox" bind:checked={autoAnalyze} style="width:auto;" />
      {i18n("auto_analyze")}
    </label>
    <p style="font-size:12px;color:var(--text-tertiary);margin-top:4px;">
      {i18n("auto_analyze_desc")}
    </p>
  </div>

  <div class="settings-group" style="margin-top:8px;">
    <button class="btn" onclick={handleAnalyze} disabled={analyzing}>
      {analyzing ? i18n("analyzing") : i18n("analyze_now")}
    </button>
    {#if analyzedCount !== null}
      <span style="margin-left:10px;font-size:13px;color:var(--text-secondary);">
        {i18n("imported_n", { n: analyzedCount })}
      </span>
    {/if}
  </div>

  <div class="settings-actions">
    <button class="btn btn-primary" onclick={handleSave}>
      {saved ? i18n("saved") : i18n("save_settings")}
    </button>
  </div>
</div>
