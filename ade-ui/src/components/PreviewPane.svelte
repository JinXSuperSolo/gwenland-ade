<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import CornersOutIcon from "phosphor-svelte/lib/CornersOutIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
  import Markdown from "../features/renderers/Markdown.svelte";
  import Mermaid from "../features/renderers/Mermaid.svelte";
  import { artifact, closeArtifact } from "../features/chat/artifact.svelte";
  import { ui } from "../shared/ui.svelte";

  const current = $derived(artifact.current);

  async function detach() {
    try {
      await invoke("open_preview_window");
      ui.previewDetached = true;
    } catch (err) {
      console.error("detach failed", err);
    }
  }
</script>

<div class="preview">
  <div class="bar" data-tauri-drag-region>
    <span class="label">{current?.title ?? "Preview"}</span>
    <div class="actions">
      <button class="icon-btn" onclick={detach} aria-label="Detach preview" title="Detach preview">
        <CornersOutIcon size={15} />
      </button>
      <button class="icon-btn" onclick={closeArtifact} aria-label="Close preview" title="Close preview">
        <XIcon size={15} />
      </button>
    </div>
  </div>

  <div class="pane-body">
    {#if !current}
      <div class="empty">
        <span class="mark"><SparkleIcon size={22} weight="fill" /></span>
        <p>Artifacts the agent renders will appear here.</p>
      </div>
    {:else if current.kind === "html"}
      <iframe
        title={current.title}
        srcdoc={current.content}
        sandbox="allow-scripts"
      ></iframe>
    {:else if current.kind === "mermaid"}
      <div class="doc"><Mermaid source={current.content} /></div>
    {:else if current.kind === "markdown"}
      <div class="doc md"><Markdown source={current.content} /></div>
    {:else}
      <pre class="doc code"><code>{current.content}</code></pre>
    {/if}
  </div>
</div>

<style>
  .preview {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
    background: var(--background);
  }

  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 8px 0 16px;
    flex-shrink: 0;
  }

  .label {
    font-size: 12px;
    font-weight: 600;
    color: var(--foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    user-select: none;
  }

  .actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }

  .icon-btn {
    display: flex;
    width: 28px;
    height: 28px;
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

  .pane-body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    animation: fade-in 0.3s ease;
  }

  iframe {
    width: 100%;
    height: 100%;
    border: none;
    background: white;
  }

  .doc {
    padding: 20px 24px;
  }
  .doc.code {
    font-family: var(--font-mono);
    font-size: 12.5px;
    line-height: 1.6;
    color: var(--foreground);
    white-space: pre;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    height: 100%;
    color: var(--muted-foreground);
    text-align: center;
    padding: 0 32px;
  }
  .empty .mark {
    color: var(--primary);
    opacity: 0.6;
  }
  .empty p {
    font-family: var(--font-sans);
    font-size: 13px;
    opacity: 0.8;
    max-width: 240px;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
