<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { searchPages, switchOrOpenTab, openNewTab, closeTab, hideSearchWindow } from "../tauri/commands";
  import type { SearchResult } from "../tauri/types";
  import EmptyState from "./EmptyState.svelte";
  import ResultList from "./ResultList.svelte";

  let { onNavigate }: { onNavigate: (v: string) => void } = $props();

  let query = $state("");
  let results: SearchResult[] = $state([]);
  let selectedIndex = $state(0);
  let loading = $state(false);
  let blurTimer: ReturnType<typeof setTimeout>;
  let inputEl: HTMLInputElement;

  async function hideWindow() {
    try {
      await hideSearchWindow();
    } catch (e) {
      console.error("Failed to hide window:", e);
    }
  }

  async function doSearch(q: string) {
    query = q;
    if (q.trim().length === 0) {
      results = [];
      return;
    }
    loading = true;
    try {
      results = await searchPages(q);
      selectedIndex = 0;
    } catch (e) {
      console.error("Search failed:", e);
      results = [];
    } finally {
      loading = false;
    }
  }

  async function handleSelect(item: SearchResult, forceNew: boolean = false) {
    // Hide the popup first so focus transfer to Chrome doesn't interfere
    hideWindow().catch(() => {});
    try {
      if (forceNew) {
        await openNewTab(item.entry.url);
      } else {
        await switchOrOpenTab(item.entry.url, false);
      }
    } catch (e) {
      console.error("Failed to open tab:", e);
    }
  }

  let tabModifier = $state(false);

  function handleKeyDown(e: KeyboardEvent) {
    // Track Tab as modifier
    if (e.key === "Tab") {
      e.preventDefault();
      if (results.length > 0 && results[selectedIndex]?.is_open) {
        tabModifier = true;
      }
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (results.length > 0 && results[selectedIndex]) {
        if (tabModifier) {
          // Tab+Enter: force new tab
          handleSelect(results[selectedIndex], true);
        } else {
          handleSelect(results[selectedIndex], e.ctrlKey || e.metaKey);
        }
      }
    } else if (e.key === "Backspace") {
      if (tabModifier && results.length > 0 && results[selectedIndex]?.is_open) {
        e.preventDefault();
        // Tab+Backspace: close the tab in browser
        closeTab(results[selectedIndex].entry.url).catch(() => {});
        // Refresh search to update is_open status
        doSearch(query);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      hideWindow().catch(() => {});
    }

    // Reset tab modifier on any non-Tab key
    if (e.key !== "Tab") {
      tabModifier = false;
    }
  }

  onMount(() => {
    const appWindow = getCurrentWindow();
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        clearTimeout(blurTimer);
        blurTimer = setTimeout(() => hideWindow(), 200);
      } else {
        clearTimeout(blurTimer);
        // Auto-focus input when window gains focus
        setTimeout(() => inputEl?.focus(), 50);
      }
    });
    // Focus on initial load
    setTimeout(() => inputEl?.focus(), 100);
    return () => {
      unlisten.then((fn) => fn());
      clearTimeout(blurTimer);
    };
  });
</script>

<div class="search-overlay" onkeydown={handleKeyDown}>
  <div class="search-container">
    <div class="search-input-wrapper">
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.35-4.35" />
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="搜索页面名称、URL 或拼音..."
        value={query}
        oninput={(e) => doSearch((e.target as HTMLInputElement).value)}
        spellcheck="false"
        autocomplete="off"
        autofocus
        bind:this={inputEl}
      />
    </div>

    {#if loading}
      <div class="empty-state">搜索中...</div>
    {:else if query.length === 0}
      <EmptyState message="输入关键词搜索..." icon="🔍" />
    {:else if results.length === 0}
      <EmptyState message="没有匹配的结果" icon="📭">
        <span class="empty-state-action" onclick={() => onNavigate("manager")}>添加新页面</span>
      </EmptyState>
    {:else}
      <ResultList
        {results}
        {selectedIndex}
        onSelect={handleSelect}
        onClose={(item) => {
          closeTab(item.entry.url).catch(() => {});
          doSearch(query);
        }}
        onForceNew={(item) => handleSelect(item, true)}
      />
    {/if}
  </div>
</div>
