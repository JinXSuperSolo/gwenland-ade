<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import ExportIcon from "phosphor-svelte/lib/ExportIcon";
  import DownloadSimpleIcon from "phosphor-svelte/lib/DownloadSimpleIcon";

  const FILES = [
    { id: "failures.md", label: "failures.md" },
    { id: "preferences.md", label: "preferences.md" },
  ] as const;

  let activeFile = $state<string>(FILES[0].id);
  let content = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let dirty = $state(false);
  let error = $state<string | null>(null);
  let savedFlash = $state(false);

  async function load(file: string) {
    loading = true;
    error = null;
    try {
      content = await invoke<string>("read_memory_file", { filename: file });
      dirty = false;
    } catch (e) {
      error = String(e);
      content = "";
    } finally {
      loading = false;
    }
  }

  async function selectFile(file: string) {
    if (file === activeFile) return;
    activeFile = file;
    await load(file);
  }

  async function save() {
    saving = true;
    error = null;
    try {
      await invoke("write_memory_file", { filename: activeFile, content });
      dirty = false;
      savedFlash = true;
      setTimeout(() => (savedFlash = false), 1500);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function exportMemory() {
    error = null;
    try {
      await invoke("export_memory");
    } catch (e) {
      error = String(e);
    }
  }

  async function importMemory() {
    error = null;
    // Overwrite warning before clobbering existing memory (GWEN-492).
    if (!confirm("Import will overwrite existing memory files with the backup's contents. Continue?")) {
      return;
    }
    try {
      const ok = await invoke<boolean>("import_memory");
      if (ok) await load(activeFile); // reflect imported content
    } catch (e) {
      error = String(e);
    }
  }

  onMount(() => load(activeFile));
</script>

<div class="memory">
  <div class="toolbar">
    <div class="tabs">
      {#each FILES as f}
        <button class="tab" class:active={f.id === activeFile} onclick={() => selectFile(f.id)}>
          {f.label}
        </button>
      {/each}
    </div>
    <div class="io">
      <button class="icon-btn" onclick={exportMemory} title="Export backup (.zip)" aria-label="Export memory">
        <ExportIcon size={16} />
      </button>
      <button class="icon-btn" onclick={importMemory} title="Import backup (.zip)" aria-label="Import memory">
        <DownloadSimpleIcon size={16} />
      </button>
    </div>
  </div>

  <div class="editor">
    {#if loading}
      <p class="muted">Loading…</p>
    {:else}
      <textarea
        bind:value={content}
        oninput={() => (dirty = true)}
        spellcheck="false"
        placeholder="This memory file is empty."
      ></textarea>
    {/if}
  </div>

  {#if error}
    <p class="error">⚠ {error}</p>
  {/if}

  <div class="footer">
    <span class="status">
      {#if savedFlash}
        <span class="saved"><CheckIcon size={13} weight="bold" /> Saved</span>
      {:else if dirty}
        Unsaved changes
      {/if}
    </span>
    <button class="save" onclick={save} disabled={!dirty || saving}>
      {saving ? "Saving…" : "Save"}
    </button>
  </div>
</div>

<style>
  .memory {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding-top: 4px;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }

  .tabs {
    display: flex;
    gap: 4px;
  }

  .tab {
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 6px);
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .tab:hover {
    background: var(--secondary);
    color: var(--foreground);
  }

  .tab.active {
    background: var(--secondary);
    color: var(--primary);
  }

  .io {
    display: flex;
    gap: 2px;
  }

  .icon-btn {
    display: flex;
    width: 30px;
    height: 30px;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 8px);
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .icon-btn:hover {
    background: color-mix(in srgb, var(--primary) 15%, transparent);
    color: var(--primary);
  }

  .editor {
    flex: 1;
    min-height: 220px;
    display: flex;
  }

  textarea {
    width: 100%;
    height: 100%;
    resize: none;
    background: var(--secondary);
    border: none;
    outline: none;
    border-radius: calc(var(--radius) - 4px);
    padding: 12px 14px;
    font-family: var(--font-mono);
    font-size: 12.5px;
    line-height: 1.6;
    color: var(--foreground);
  }

  textarea::placeholder {
    color: var(--muted-foreground);
    opacity: 0.6;
  }

  .error {
    margin-top: 8px;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--destructive);
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 12px;
  }

  .status {
    font-family: var(--font-sans);
    font-size: 11.5px;
    color: var(--muted-foreground);
  }

  .saved {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--primary);
  }

  .save {
    background: var(--primary);
    border: none;
    color: var(--primary-foreground);
    font-family: var(--font-sans);
    font-size: 13px;
    padding: 7px 18px;
    border-radius: calc(var(--radius) - 6px);
    cursor: pointer;
    transition: background 0.15s, opacity 0.15s;
  }

  .save:hover:not(:disabled) {
    background: color-mix(in srgb, var(--primary) 85%, white);
  }

  .save:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .muted {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--muted-foreground);
  }
</style>
