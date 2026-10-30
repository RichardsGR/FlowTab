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

  function getDomain(url: string): string {
    try {
      return new URL(url).hostname.replace("www.", "");
    } catch {
      return url;
    }
  }
</script>

<li
  class="result-item"
  class:selected
  onclick={() => onSelect(false)}
  onauxclick={(e) => {
    if (e.button === 1 || e.metaKey || e.ctrlKey) {
      e.preventDefault();
      onSelect(true);
    }
  }}
  role="option"
  aria-selected={selected}
>
  <div class="result-item-icon">
    {item.entry.name.charAt(0)}
  </div>
  <div class="result-item-content">
    <div class="result-item-name">{item.entry.name}</div>
    <div class="result-item-url">{getDomain(item.entry.url)}</div>
  </div>
  {#if item.is_open}
    <span class="result-item-badge badge-open">{i18n("open")}</span>
    {#if selected}
      <button class="result-item-action" title={i18n("close_tab_title")}
        onclick={(e) => { e.stopPropagation(); onClose?.(item); }}>
        ✕
      </button>
      <button class="result-item-action primary" title={i18n("dup_tab_title")}
        onclick={(e) => { e.stopPropagation(); onForceNew?.(item); }}>
        +
      </button>
    {/if}
  {:else if selected}
    <span class="result-item-action primary" title={i18n("open_title")}>
      ⏎
    </span>
  {/if}
</li>
