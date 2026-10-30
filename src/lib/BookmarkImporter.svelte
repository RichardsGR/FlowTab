<script lang="ts">
  import { importBookmarks } from "../tauri/commands";
  import { i18n, type Lang } from "../tauri/i18n";

  let {
    lang,
    onBack,
  }: { lang: Lang; onBack: () => void } = $props();

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
    <button class="btn" onclick={onBack}>{i18n(lang, "back")}</button>
    <h2 style="margin-bottom:0;">{i18n(lang, "import_bookmarks")}</h2>
  </div>

  <p style="color:var(--text-secondary);font-size:14px;margin-bottom:16px;">
    {i18n(lang, "import_desc")}
  </p>

  <button class="btn btn-primary" onclick={handleImport} disabled={importing}>
    {importing ? i18n(lang, "importing") : i18n(lang, "start_import")}
  </button>

  {#if importedCount !== null}
    <div class="importer-status">
      {i18n(lang, "import_success", { n: importedCount })}
    </div>
  {/if}

  {#if error}
    <div class="importer-status" style="color:rgb(255,59,48);">
      {i18n(lang, "import_failed", { e: error })}
    </div>
  {/if}
</div>
