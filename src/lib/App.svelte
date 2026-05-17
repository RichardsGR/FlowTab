<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getSetting, analyzeHistory } from "../tauri/commands";
  import "../styles/global.css";
  import "../styles/search.css";
  import SearchPopup from "./SearchPopup.svelte";
  import PageIndexManager from "./PageIndexManager.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";
  import BookmarkImporter from "./BookmarkImporter.svelte";

  let view: "search" | "manager" | "settings" | "import" = $state("search");

  function applyTheme(t: string) {
    const html = document.documentElement;
    html.classList.remove("theme-dark", "theme-light");
    let resolved = t;
    if (t === "system") {
      resolved = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
    html.classList.add(resolved === "light" ? "theme-light" : "theme-dark");
  }

  onMount(async () => {
    // Load saved theme
    try {
      const saved = await getSetting("theme");
      applyTheme(saved ?? "system");

      // Auto-analyze history if enabled
      const autoAnalyze = await getSetting("auto_analyze");
      if (autoAnalyze === "true") {
        analyzeHistory().then(n => {
          if (n > 0) console.log(`Auto-imported ${n} pages from history`);
        }).catch(() => {});
      }
    } catch { /* use default dark */ }

    const unlisten = listen<string>("navigate", (event) => {
      view = event.payload as "search" | "manager" | "settings" | "import";
    });
    return () => { unlisten.then(fn => fn()); };
  });
</script>

{#if view === "search"}
  <SearchPopup onNavigate={(v) => (view = v)} />
{:else if view === "manager"}
  <PageIndexManager onBack={() => (view = "search")} />
{:else if view === "settings"}
  <SettingsPanel onBack={() => (view = "search")} />
{:else if view === "import"}
  <BookmarkImporter onBack={() => (view = "search")} />
{/if}
