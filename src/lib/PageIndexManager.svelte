<script lang="ts">
  import { onMount } from "svelte";
  import { listPages, addPage, updatePage, deletePage, cleanDuplicatePages } from "../tauri/commands";
  import type { PageEntry } from "../tauri/types";

  let { onBack }: { onBack: () => void } = $props();

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
    <button class="btn" onclick={onBack}>← 返回</button>
    <h2 style="margin-bottom:0;">页面索引管理</h2>
    <div style="flex:1"></div>
    <button class="btn btn-danger" onclick={handleClean} disabled={cleaning}>
      {cleaning ? "清理中..." : "清理重复"}
    </button>
    {#if cleanedCount !== null}
      <span style="font-size:13px;color:var(--text-secondary);">已清理 {cleanedCount} 条</span>
    {/if}
  </div>

  <div class="page-form">
    <div class="page-form-row">
      <input type="text" placeholder="页面名称 (如: Rust官网)" bind:value={newName} />
      <input type="text" placeholder="URL (如: https://rust-lang.org)" bind:value={newUrl} />
      <button class="btn btn-primary" onclick={handleAdd}>添加</button>
    </div>
  </div>

  <ul class="page-list">
    {#each pages as page (page.id)}
      <li class="page-list-item">
        {#if editingId === page.id}
          <input type="text" bind:value={editName} style="flex:1" />
          <input type="text" bind:value={editUrl} style="flex:2" />
          <button class="btn btn-primary" onclick={() => handleUpdate(page.id)}>保存</button>
          <button class="btn" onclick={() => (editingId = 0)}>取消</button>
        {:else}
          <span class="page-list-item-name">{page.name}</span>
          <span class="page-list-item-url">{page.url}</span>
          <div class="page-list-item-actions">
            <button class="btn" onclick={() => startEdit(page)}>编辑</button>
            <button class="btn btn-danger" onclick={() => handleDelete(page.id)}>删除</button>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
</div>
