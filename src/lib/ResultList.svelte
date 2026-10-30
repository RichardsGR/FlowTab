<script lang="ts">
  import type { SearchResult } from "../tauri/types";
  import ResultItem from "./ResultItem.svelte";

  import type { Lang } from "../tauri/i18n";

  let {
    results,
    selectedIndex,
    onSelect,
    onClose,
    onForceNew,
    lang,
  }: {
    results: SearchResult[];
    selectedIndex: number;
    onSelect: (item: SearchResult, forceNew: boolean) => void;
    onClose?: (item: SearchResult) => void;
    onForceNew?: (item: SearchResult) => void;
    lang: Lang;
  } = $props();

  let listEl: HTMLUListElement;

  $effect(() => {
    const idx = selectedIndex;
    if (!listEl || idx < 0 || idx >= listEl.children.length) return;

    requestAnimationFrame(() => {
      const item = listEl.children[idx] as HTMLElement;
      const list = listEl;
      const itemTop = item.offsetTop - list.offsetTop;
      const listTop = list.scrollTop;
      const listBottom = listTop + list.clientHeight;

      if (itemTop < listTop) {
        list.scrollTo({ top: itemTop - 8, behavior: "smooth" });
      } else if (item.offsetTop + item.offsetHeight > listBottom) {
        list.scrollTo({ top: item.offsetTop + item.offsetHeight - list.clientHeight + 8, behavior: "smooth" });
      }
    });
  });
</script>

<ul class="result-list" bind:this={listEl}>
  {#each results as item, i}
    <ResultItem
      {item}
      {lang}
      selected={i === selectedIndex}
      onSelect={(forceNew) => onSelect(item, forceNew)}
      {onClose}
      {onForceNew}
    />
  {/each}
</ul>
