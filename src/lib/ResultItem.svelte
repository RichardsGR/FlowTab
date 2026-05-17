<script lang="ts">
  import type { SearchResult } from "../tauri/types";

  let {
    item,
    selected,
    onSelect,
  }: {
    item: SearchResult;
    selected: boolean;
    onSelect: (forceNew: boolean) => void;
  } = $props();

  function getDomain(url: string): string {
    try {
      return new URL(url).hostname.replace("www.", "");
    } catch {
      return url;
    }
  }

  // Only show badge if the tab is already open
  let showBadge = $derived(item.is_open);
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
  {#if showBadge}
    <span class="result-item-badge badge-open">已打开</span>
  {/if}
  <span class="result-item-shortcut">
    {selected ? "⏎" : ""}
  </span>
</li>
