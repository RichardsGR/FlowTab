<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { searchPages, prefetchOpenTabs, switchOrOpenTab, openNewTab, closeTab, hideSearchWindow } from "../tauri/commands";
  import type { SearchResult } from "../tauri/types";
  import { i18n, type Lang } from "../tauri/i18n";
  import EmptyState from "./EmptyState.svelte";
  import ResultList from "./ResultList.svelte";

  let { lang, onNavigate }: { lang: Lang; onNavigate: (v: string) => void } = $props();

  let query = $state("");
  let results: SearchResult[] = $state([]);
  let selectedIndex = $state(0);
  let loading = $state(false);
  let automationError = $state<string | null>(null);
  let blurTimer: ReturnType<typeof setTimeout>;
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;
  let searchSeq = 0;
  let shouldResetOnFocus = true;
  let inputEl: HTMLInputElement;

  function resetSearch() {
    clearTimeout(debounceTimer);
    searchSeq++;
    query = "";
    results = [];
    selectedIndex = 0;
    loading = false;
    automationError = null;
  }

  async function hideWindow() {
    shouldResetOnFocus = true;
    try {
      await hideSearchWindow();
    } catch (e) {
      console.error("Failed to hide window:", e);
    }
  }

  function handleInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    query = value;
    clearTimeout(debounceTimer);

    if (value.trim().length === 0) {
      results = [];
      loading = false;
      return;
    }

    debounceTimer = setTimeout(() => doSearch(value), 200);
  }

  async function doSearch(q: string) {
    clearTimeout(debounceTimer);
    const seq = ++searchSeq;
    query = q;

    if (q.trim().length === 0) {
      results = [];
      loading = false;
      return;
    }

    loading = true;
    try {
      const next = await searchPages(q);
      if (seq !== searchSeq) return;
      results = next;
      selectedIndex = 0;
    } catch (e) {
      if (seq !== searchSeq) return;
      console.error("Search failed:", e);
      results = [];
    } finally {
      if (seq === searchSeq) loading = false;
    }
  }

  function handleActionError(e: unknown) {
    const msg = String(e);
    if (msg.includes("AUTOMATION_DENIED")) {
      const browser = msg.split(":")[1]?.trim() || "浏览器";
      automationError = i18n(lang, "automation_denied", { browser });
    } else {
      console.error("Browser action failed:", e);
    }
  }

  async function handleSelect(item: SearchResult, forceNew: boolean = false) {
    try {
      if (forceNew) {
        await openNewTab(item.entry.url);
      } else {
        await switchOrOpenTab(item.entry.url, false);
      }
      automationError = null;
      // Hide only after the browser action succeeds, so permission errors stay visible
      hideWindow().catch(() => {});
    } catch (e) {
      handleActionError(e);
    }
  }

  async function handleClose(item: SearchResult) {
    try {
      await closeTab(item.entry.url);
      automationError = null;
      doSearch(query);
    } catch (e) {
      handleActionError(e);
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
        handleClose(results[selectedIndex]);
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
        shouldResetOnFocus = true;
        clearTimeout(blurTimer);
        blurTimer = setTimeout(() => hideWindow(), 200);
      } else {
        clearTimeout(blurTimer);
        // Every time the popup opens, start with a clean search
        if (shouldResetOnFocus) {
          resetSearch();
          shouldResetOnFocus = false;
        }
        // Warm the open-tab cache so the first search is instant
        prefetchOpenTabs().catch(() => {});
        // Auto-focus input when window gains focus
        setTimeout(() => inputEl?.focus(), 50);
      }
    });
    // Rust emits this every time it shows the window; focus cannot be
    // guaranteed by the window focus event alone.
    const unlistenShown = listen("search-shown", () => {
      resetSearch();
      shouldResetOnFocus = false;
      setTimeout(() => inputEl?.focus(), 30);
    });
    // Focus on initial load
    setTimeout(() => inputEl?.focus(), 100);
    return () => {
      unlisten.then((fn) => fn());
      unlistenShown.then((fn) => fn());
      clearTimeout(blurTimer);
      clearTimeout(debounceTimer);
    };
  });
</script>

<div
  class="search-overlay"
  onkeydown={handleKeyDown}
  onclick={(e) => {
    const target = e.target as HTMLElement;
    if (!target.closest("button, input, select, a")) {
      inputEl?.focus();
    }
  }}
>
  <div class="search-container">
    <div class="search-input-wrapper">
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-4-4" />
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder={i18n(lang, "search_placeholder")}
        value={query}
        oninput={handleInput}
        spellcheck="false"
        autocomplete="off"
        autofocus
        bind:this={inputEl}
      />
      <button class="esc-key" onclick={() => hideWindow().catch(() => {})} aria-label={i18n(lang, "hint_close")}>esc</button>
    </div>

    {#if automationError}
      <div class="automation-banner" role="alert">
        <span>{automationError}</span>
        <button
          class="automation-banner-close"
          onclick={() => (automationError = null)}
          aria-label={i18n(lang, "hint_close")}
        >✕</button>
      </div>
    {/if}

    <div class="search-body">
      {#if query.length === 0}
        <EmptyState message={i18n(lang, "type_to_search")}>
          {#snippet icon()}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
              <circle cx="11" cy="11" r="6.5" />
              <path d="m19 19-3.5-3.5" />
              <path d="M8.5 11h5M11 8.5v5" opacity=".45" />
            </svg>
          {/snippet}
        </EmptyState>
      {:else if results.length === 0 && loading}
        <div class="search-skeleton" aria-hidden="true">
          {#each [0, 1, 2] as row (row)}
            <div class="skeleton-row">
              <span class="skeleton-icon"></span>
              <span class="skeleton-lines">
                <span class="skeleton-line wide"></span>
                <span class="skeleton-line narrow"></span>
              </span>
            </div>
          {/each}
        </div>
      {:else if results.length === 0}
        <EmptyState message={i18n(lang, "no_results")}>
          {#snippet icon()}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
              <circle cx="11" cy="11" r="6.5" />
              <path d="m19 19-3.5-3.5" />
              <path d="m9 13 4-4M9 9l4 4" opacity=".45" />
            </svg>
          {/snippet}
          <span class="empty-state-action" onclick={() => onNavigate("manager")}>{i18n(lang, "add_page")}</span>
        </EmptyState>
      {:else}
        <ResultList
          {results}
          {selectedIndex}
          {lang}
          onSelect={handleSelect}
          onClose={handleClose}
          onForceNew={(item) => handleSelect(item, true)}
        />
      {/if}
    </div>

    <div class="search-footer">
      <span class="hint"><kbd>↑</kbd><kbd>↓</kbd>{i18n(lang, "hint_navigate")}</span>
      <span class="hint"><kbd>↵</kbd>{i18n(lang, "hint_open")}</span>
      <span class="hint"><kbd>tab</kbd><kbd>↵</kbd>{i18n(lang, "hint_duplicate")}</span>
      <span class="hint"><kbd>tab</kbd><kbd>⌫</kbd>{i18n(lang, "hint_close")}</span>
    </div>
  </div>
</div>
