<script lang="ts">
  import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
  import Markdown from "../features/renderers/Markdown.svelte";
  import Mermaid from "../features/renderers/Mermaid.svelte";
  import { artifact } from "../features/chat/artifact.svelte";

  // The detached preview is its own webview (separate JS runtime). It renders
  // the same artifact store. Reattach is Rust-driven (see lib.rs / App.svelte).
  const current = $derived(artifact.current);
</script>

<div class="detached-root">
  <div class="bar" data-tauri-drag-region>
    <span class="label">{current?.title ?? "ADE Preview"}</span>
  </div>
  <div class="body">
    {#if !current}
      <div class="empty">
        <span class="mark"><SparkleIcon size={22} weight="fill" /></span>
        <p>Artifacts the agent renders will appear here.</p>
      </div>
    {:else if current.kind === "html"}
      <iframe title={current.title} srcdoc={current.content} sandbox="allow-scripts"></iframe>
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
  .detached-root {
    height: 100vh;
    width: 100vw;
    background: var(--background);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .bar {
    height: 36px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    padding: 0 16px;
  }
  .label {
    font-size: 12px;
    font-weight: 600;
    color: var(--foreground);
    user-select: none;
  }
  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
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
  }
  .empty .mark { color: var(--primary); opacity: 0.6; }
  .empty p { font-family: var(--font-sans); font-size: 13px; opacity: 0.8; }
</style>
