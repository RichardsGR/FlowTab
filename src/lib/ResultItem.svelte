<script lang="ts">
  import type { SearchResult } from "../tauri/types";
  import { i18n, type Lang } from "../tauri/i18n";

  let {
    item,
    selected,
    onSelect,
    onClose,
    onForceNew,
    lang,
  }: {
    item: SearchResult;
    selected: boolean;
    onSelect: (forceNew: boolean) => void;
    onClose?: (item: SearchResult) => void;
    onForceNew?: (item: SearchResult) => void;
    lang: Lang;
  } = $props();

  function getDisplayUrl(url: string): string {
    try {
      const parsed = new URL(url);
      const host =
        parsed.hostname.replace(/^www\./, "") +
        (parsed.port ? `:${parsed.port}` : "");
      const path = parsed.pathname.replace(/\/$/, "");
      return `${host}${path}${parsed.search}`;
    } catch {
      return url;
    }
  }

  function tileHue(text: string): number {
    let hue = 0;
    for (const char of text) {
      hue = (hue * 31 + char.charCodeAt(0)) % 360;
    }
    return hue;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<li
  class="result-item"
  class:selected
  style={`--tile-hue:${tileHue(getDisplayUrl(item.entry.url))}`}
  onclick={() => onSelect(false)}
  onauxclick={(e) => {
    if (e.button === 1 || e.metaKey || e.ctrlKey) {
      e.preventDefault();
      onSelect(true);
    }
  }}
  role="option"
  aria-selected={selected}
  title={item.entry.url}
>
  <span class="result-item-indicator"></span>
  <span class="result-item-icon">{item.entry.name.charAt(0).toUpperCase()}</span>
  <span class="result-item-content">
    <span class="result-item-name">{item.entry.name}</span>
    <span class="result-item-url">{getDisplayUrl(item.entry.url)}</span>
  </span>
  {#if item.is_open}
    <span class="result-item-badge">{i18n(lang, "open")}</span>
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
    <span class="result-item-enter" title={i18n(lang, "open_title")}>⏎</span>
  {/if}
</li>
