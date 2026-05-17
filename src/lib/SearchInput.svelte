<script lang="ts">
  import { onMount } from "svelte";

  let {
    query,
    onSearch,
    onNavigate,
  }: {
    query: string;
    onSearch: (q: string) => void;
    onNavigate: (v: string) => void;
  } = $props();

  let inputEl: HTMLInputElement;

  onMount(() => {
    inputEl?.focus();
  });

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    onSearch(target.value);
  }
</script>

<div class="search-input-wrapper">
  <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.35-4.35" />
  </svg>
  <input
    bind:this={inputEl}
    type="text"
    class="search-input"
    placeholder="搜索页面名称、URL 或拼音..."
    value={query}
    oninput={handleInput}
    spellcheck="false"
    autocomplete="off"
  />
  <button class="search-hint" onclick={() => onNavigate("settings")} title="设置">
    ⚙
  </button>
</div>
