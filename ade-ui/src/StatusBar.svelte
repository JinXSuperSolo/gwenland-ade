<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import BrainIcon from "phosphor-svelte/lib/BrainIcon";
  import FolderSimpleIcon from "phosphor-svelte/lib/FolderSimpleIcon";
  import { chat } from "./conversation.svelte";
  import { openSettings } from "./ui.svelte";

  let workspace = $state<string | null>(null);

  onMount(async () => {
    workspace = await invoke<string | null>("get_workspace").catch(() => null);
  });

  // Show only the trailing folder name to keep the bar thin.
  let workspaceName = $derived(
    workspace ? workspace.replace(/[/\\]+$/, "").split(/[/\\]/).pop() : null,
  );

  let modelLabel = $derived(chat.modelId || chat.providerId || "no model");
</script>

<div class="statusbar">
  <button
    class="cell"
    title={workspace ?? "No workspace selected"}
    onclick={() => invoke("pick_workspace").then((p) => (workspace = p as string | null)).catch(() => {})}
  >
    <FolderSimpleIcon size={12} />
    <span>{workspaceName ?? "No workspace"}</span>
  </button>

  <div class="spacer"></div>

  <button class="cell" title="Open memory (Ctrl+M)" onclick={() => openSettings("memory")}>
    <BrainIcon size={12} />
    <span>Memory</span>
  </button>

  <div class="cell static" title="Active model">
    <span>{modelLabel}</span>
  </div>
</div>

<style>
  .statusbar {
    height: 28px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 8px;
    background: var(--background);
    font-family: var(--font-sans);
  }

  .spacer {
    flex: 1;
  }

  .cell {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 20px;
    padding: 0 8px;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 10px);
    color: var(--muted-foreground);
    font-size: 11px;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .cell.static {
    cursor: default;
  }

  .cell:not(.static):hover {
    background: var(--secondary);
    color: var(--foreground);
  }

  .cell span {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
