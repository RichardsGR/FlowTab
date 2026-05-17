<script lang="ts">
  import { onMount } from "svelte";
  import { getSetting, setSetting, analyzeHistory } from "../tauri/commands";

  let { onBack }: { onBack: () => void } = $props();

  let hotkey = $state("Ctrl+K");
  let activeBrowser = $state("chrome");
  let theme = $state("system");
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
    if (resolved === "light") {
      html.classList.add("theme-light");
    } else {
      html.classList.add("theme-dark");
    }
  }

  async function loadSettings() {
    try {
      hotkey = (await getSetting("global_hotkey")) ?? "Ctrl+K";
      activeBrowser = (await getSetting("active_browser")) ?? "chrome";
      theme = (await getSetting("theme")) ?? "system";
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
      await setSetting("auto_analyze", autoAnalyze ? "true" : "false");
      applyTheme(theme);
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
    <button class="btn" onclick={onBack}>← 返回</button>
    <h2 style="margin-bottom:0;">设置</h2>
  </div>

  <div class="settings-group">
    <label>全局快捷键</label>
    <input type="text" bind:value={hotkey} placeholder="如: Ctrl+K" />
  </div>

  <div class="settings-group">
    <label>默认浏览器</label>
    <select bind:value={activeBrowser}>
      <option value="chrome">Google Chrome</option>
      <option value="edge">Microsoft Edge</option>
      <option value="firefox">Firefox</option>
    </select>
  </div>

  <div class="settings-group">
    <label>主题</label>
    <select bind:value={theme}>
      <option value="system">跟随系统</option>
      <option value="dark">深色</option>
      <option value="light">浅色</option>
    </select>
  </div>

  <div class="settings-group" style="border-top:1px solid var(--border);padding-top:16px;margin-top:16px;">
    <label style="display:flex;align-items:center;gap:8px;">
      <input type="checkbox" bind:checked={autoAnalyze} style="width:auto;" />
      自动分析浏览记录
    </label>
    <p style="font-size:12px;color:var(--text-tertiary);margin-top:4px;">
      读取浏览器历史记录，将高频访问的网站自动添加到搜索索引（访问次数 ≥ 5）
    </p>
  </div>

  <div class="settings-group" style="margin-top:8px;">
    <button class="btn" onclick={handleAnalyze} disabled={analyzing}>
      {analyzing ? "分析中..." : "立即分析浏览记录"}
    </button>
    {#if analyzedCount !== null}
      <span style="margin-left:10px;font-size:13px;color:var(--text-secondary);">
        已导入 {analyzedCount} 个高频页面
      </span>
    {/if}
  </div>

  <div class="settings-actions">
    <button class="btn btn-primary" onclick={handleSave}>
      {saved ? "已保存 ✓" : "保存设置"}
    </button>
  </div>
</div>
