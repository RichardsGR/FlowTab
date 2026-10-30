<script lang="ts">
  import { onMount } from "svelte";
  import { getSetting, setSetting, analyzeHistory } from "../tauri/commands";
  import { i18n, LANGUAGES, normalizeLang, type Lang } from "../tauri/i18n";

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
      language = normalizeLang(await getSetting("language"));
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

<div class="page-manager settings-view">
  <header class="settings-header">
    <button class="icon-button" onclick={onBack} aria-label={i18n(lang, "back")}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <path d="m15 18-6-6 6-6" />
      </svg>
    </button>
    <h2>{i18n(lang, "settings")}</h2>
  </header>

  <section class="settings-section">
    <h3 class="settings-section-title">{i18n(lang, "section_general")}</h3>

    <div class="settings-row">
      <span class="settings-row-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <rect x="3" y="6" width="18" height="12" rx="2.5" />
          <path d="M7 10h.01M11 10h.01M15 10h.01M7 14h6" />
        </svg>
      </span>
      <span class="settings-row-label">{i18n(lang, "global_shortcut")}</span>
      <input class="settings-control shortcut-input" type="text" bind:value={hotkey} placeholder="Ctrl+K" />
    </div>

    <div class="settings-row">
      <span class="settings-row-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <rect x="3" y="4" width="18" height="16" rx="2.5" />
          <path d="M3 9h18" />
        </svg>
      </span>
      <span class="settings-row-label">{i18n(lang, "default_browser")}</span>
      <select class="settings-control" bind:value={activeBrowser}>
        <option value="chrome">Google Chrome</option>
        <option value="safari">Safari</option>
        <option value="firefox">Firefox</option>
      </select>
    </div>
    {#if activeBrowser === "safari"}
      <p class="settings-hint">{i18n(lang, "safari_first_use_hint")}</p>
    {/if}

    <div class="settings-row">
      <span class="settings-row-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <circle cx="12" cy="12" r="4" />
          <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
        </svg>
      </span>
      <span class="settings-row-label">{i18n(lang, "theme")}</span>
      <select class="settings-control" bind:value={theme}>
        <option value="system">{i18n(lang, "theme_system")}</option>
        <option value="dark">{i18n(lang, "theme_dark")}</option>
        <option value="light">{i18n(lang, "theme_light")}</option>
      </select>
    </div>

    <div class="settings-row">
      <span class="settings-row-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <circle cx="12" cy="12" r="8" />
          <path d="M4 12h16M12 4c2.2 2.3 3.3 5 3.3 8S14.2 17.7 12 20c-2.2-2.3-3.3-5-3.3-8S9.8 6.3 12 4Z" />
        </svg>
      </span>
      <span class="settings-row-label">{i18n(lang, "language")}</span>
      <select class="settings-control" bind:value={language}>
        {#each LANGUAGES as l (l.code)}
          <option value={l.code}>{l.label}</option>
        {/each}
      </select>
    </div>
  </section>

  <section class="settings-section">
    <h3 class="settings-section-title">{i18n(lang, "section_history")}</h3>

    <div class="settings-row settings-row-stacked">
      <span class="settings-row-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <circle cx="12" cy="12" r="8" />
          <path d="M12 8v4l2.5 1.5" />
        </svg>
      </span>
      <span class="settings-row-text">
        <span class="settings-row-label">{i18n(lang, "auto_analyze")}</span>
        <span class="settings-row-desc">{i18n(lang, "auto_analyze_desc")}</span>
        <span class="settings-inline-action">
          <button class="btn btn-secondary btn-sm" onclick={handleAnalyze} disabled={analyzing}>
            {analyzing ? i18n(lang, "analyzing") : i18n(lang, "analyze_now")}
          </button>
          {#if analyzedCount !== null}
            <span class="settings-status">{i18n(lang, "imported_n", { n: analyzedCount })}</span>
          {/if}
        </span>
      </span>
      <button
        class="switch"
        class:on={autoAnalyze}
        role="switch"
        aria-checked={autoAnalyze}
        aria-label={i18n(lang, "auto_analyze")}
        onclick={() => (autoAnalyze = !autoAnalyze)}
      >
        <span class="switch-knob"></span>
      </button>
    </div>
  </section>

  <footer class="settings-footer">
    <button class="btn btn-primary settings-save" onclick={handleSave}>
      {saved ? i18n(lang, "saved") : i18n(lang, "save_settings")}
    </button>
  </footer>
</div>
