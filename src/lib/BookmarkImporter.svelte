<script lang="ts">
  import { importBookmarks } from "../tauri/commands";

  let { onBack }: { onBack: () => void } = $props();

  let importing = $state(false);
  let importedCount = $state<number | null>(null);
  let error = $state<string | null>(null);

  async function handleImport() {
    importing = true;
    importedCount = null;
    error = null;
    try {
      const count = await importBookmarks();
      importedCount = count;
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
    }
  }
</script>

<div class="page-manager">
  <div style="display:flex;align-items:center;gap:12px;margin-bottom:16px;">
    <button class="btn" onclick={onBack}>← 返回</button>
    <h2 style="margin-bottom:0;">导入浏览器书签</h2>
  </div>

  <p style="color:var(--text-secondary);font-size:14px;margin-bottom:16px;">
    从 Chrome、Edge 等浏览器的书签中导入页面。重复的 URL 将被自动跳过。
  </p>

  <button class="btn btn-primary" onclick={handleImport} disabled={importing}>
    {importing ? "导入中..." : "开始导入"}
  </button>

  {#if importedCount !== null}
    <div class="importer-status">
      成功导入 {importedCount} 个书签。
    </div>
  {/if}

  {#if error}
    <div class="importer-status" style="color:rgb(255,59,48);">
      导入失败: {error}
    </div>
  {/if}
</div>
