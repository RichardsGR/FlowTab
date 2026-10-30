<script lang="ts">
  import { onMount } from "svelte";
  import { listPages, addPage, updatePage, deletePage, cleanDuplicatePages } from "../tauri/commands";
  import { i18n, type Lang } from "../tauri/i18n";
  import type { PageEntry } from "../tauri/types";

  let {
    lang,
    onBack,
  }: { lang: Lang; onBack: () => void } = $props();

  let pages: PageEntry[] = $state([]);
  let newName = $state("");
  let newUrl = $state("");
  let editingId = $state(0);
  let editName = $state("");
  let editUrl = $state("");
  let cleaning = $state(false);
  let cleanedCount = $state<number | null>(null);

  async function handleClean() {
    cleaning = true;
    cleanedCount = null;
    try {
      cleanedCount = await cleanDuplicatePages();
      if (cleanedCount > 0) await loadPages();
    } catch (e) {
      console.error("Clean failed:", e);
    } finally {
      cleaning = false;
    }
  }

  async function loadPages() {
    try {
      pages = await listPages();
    } catch (e) {
      console.error("Failed to load pages:", e);
    }
  }

  async function handleAdd() {
    if (!newName.trim() || !newUrl.trim()) return;
    try {
      await addPage(newName.trim(), newUrl.trim());
      newName = "";
      newUrl = "";
      await loadPages();
    } catch (e) {
      console.error("Failed to add page:", e);
    }
  }

  async function handleUpdate(id: number) {
    if (!editName.trim() || !editUrl.trim()) return;
    try {
      await updatePage(id, editName.trim(), editUrl.trim());
      editingId = 0;
      await loadPages();
    } catch (e) {
      console.error("Failed to update page:", e);
    }
  }

  async function handleDelete(id: number) {
    try {
      await deletePage(id);
      await loadPages();
    } catch (e) {
      console.error("Failed to delete page:", e);
    }
  }

  function startEdit(page: PageEntry) {
    editingId = page.id;
    editName = page.name;
    editUrl = page.url;
  }

  onMount(() => {
    loadPages();
  });
</script>

<div class="page-manager">
  <div style="display:flex;align-items:center;gap:12px;margin-bottom:16px;">
    <button class="btn" onclick={onBack}>{i18n(lang, "back")}</button>
    <h2 style="margin-bottom:0;">{i18n(lang, "page_index")}</h2>
    <div style="flex:1"></div>
    <button class="btn btn-danger" onclick={handleClean} disabled={cleaning}>
      {cleaning ? i18n(lang, "cleaning") : i18n(lang, "clean_dupes")}
    </button>
    {#if cleanedCount !== null}
      <span style="font-size:13px;color:var(--text-secondary);">{i18n(lang, "removed_n", { n: cleanedCount })}</span>
    {/if}
  </div>

  <div class="page-form">
    <div class="page-form-row">
      <input type="text" placeholder={i18n(lang, "page_name_ph")} bind:value={newName} />
      <input type="text" placeholder={i18n(lang, "url_ph")} bind:value={newUrl} />
      <button class="btn btn-primary" onclick={handleAdd}>{i18n(lang, "add")}</button>
    </div>
  </div>

  <ul class="page-list">
    {#each pages as page (page.id)}
      <li class="page-list-item">
        {#if editingId === page.id}
          <input type="text" bind:value={editName} style="flex:1" />
          <input type="text" bind:value={editUrl} style="flex:2" />
          <button class="btn btn-primary" onclick={() => handleUpdate(page.id)}>{i18n(lang, "save")}</button>
          <button class="btn" onclick={() => (editingId = 0)}>{i18n(lang, "cancel")}</button>
        {:else}
          <span class="page-list-item-name">{page.name}</span>
          <span class="page-list-item-url">{page.url}</span>
          <div class="page-list-item-actions">
            <button class="btn" onclick={() => startEdit(page)}>{i18n(lang, "edit")}</button>
            <button class="btn btn-danger" onclick={() => handleDelete(page.id)}>{i18n(lang, "del")}</button>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
</div>
