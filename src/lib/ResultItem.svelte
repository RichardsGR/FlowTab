<script lang="ts">
  import type { SearchResult } from "../tauri/types";

  let {
    item,
    selected,
    onSelect,
    onClose,
    onForceNew,
  }: {
    item: SearchResult;
    selected: boolean;
    onSelect: (forceNew: boolean) => void;
    onClose?: (item: SearchResult) => void;
    onForceNew?: (item: SearchResult) => void;
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
    <span class="result-item-badge badge-open">已打开</span>
    {#if selected}
      <button class="result-item-action" title="关闭标签页 (Tab+⌫)"
        onclick={(e) => { e.stopPropagation(); onClose?.(item); }}>
        ✕
      </button>
      <button class="result-item-action primary" title="新建相同标签页 (Tab+⏎)"
        onclick={(e) => { e.stopPropagation(); onForceNew?.(item); }}>
        +
      </button>
    {/if}
  {:else if selected}
    <span class="result-item-action primary" title="打开 (⏎)">
      ⏎
    </span>
  {/if}
</li>
