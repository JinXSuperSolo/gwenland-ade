<script lang="ts">
  import { renderMermaid } from "./mermaid";

  let { source }: { source: string } = $props();

  const result = $derived(renderMermaid(source));
</script>

<div class="mermaid">
  {#if "svg" in result}
    <div class="canvas">{@html result.svg}</div>
  {:else}
    <div class="fallback">
      <p class="note">⬡ {result.error}</p>
      <pre><code>{source}</code></pre>
    </div>
  {/if}
</div>

<style>
  .mermaid {
    /* Graph palette, resolved from the app's design tokens. */
    --md-graph-stroke: color-mix(in srgb, var(--primary) 55%, var(--border));
    --md-graph-node: color-mix(in srgb, var(--primary) 8%, var(--card));
    --md-graph-text: var(--foreground);
    --md-graph-edge: var(--muted-foreground);
    --md-graph-accent: var(--primary);
    --md-graph-bg: var(--card);

    margin: 0.9em 0;
    padding: 14px;
    background: var(--secondary);
    border-radius: calc(var(--radius) - 4px);
    overflow-x: auto;
  }

  .canvas {
    display: flex;
    justify-content: center;
  }

  .canvas :global(svg) {
    max-width: 100%;
    height: auto;
  }

  .fallback .note {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--muted-foreground);
    margin-bottom: 8px;
  }

  .fallback pre {
    overflow-x: auto;
  }

  .fallback code {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--foreground);
    white-space: pre;
  }
</style>
