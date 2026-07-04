<script lang="ts">
  import type { Inline } from "./markdown";
  import Self from "./MarkdownInline.svelte";
  import Katex from "./Katex.svelte";

  let { nodes }: { nodes: Inline[] } = $props();
</script>

{#each nodes as n}
  {#if n.type === "text"}{n.value}
  {:else if n.type === "strong"}<strong><Self nodes={n.children} /></strong>
  {:else if n.type === "em"}<em><Self nodes={n.children} /></em>
  {:else if n.type === "del"}<del><Self nodes={n.children} /></del>
  {:else if n.type === "code"}<code class="inline-code">{n.value}</code>
  {:else if n.type === "math"}<Katex source={n.value} />
  {:else if n.type === "link"}<a href={n.href} target="_blank" rel="noopener noreferrer"><Self nodes={n.children} /></a>
  {:else if n.type === "image"}<img src={n.src} alt={n.alt} />
  {:else if n.type === "break"}<br />
  {/if}
{/each}

<style>
  .inline-code {
    font-family: var(--font-mono);
    font-size: 0.88em;
    background: var(--secondary);
    padding: 1px 5px;
    border-radius: 5px;
    color: var(--foreground);
    white-space: pre-wrap;
  }

  a {
    color: var(--primary);
    text-decoration: none;
  }
  a:hover {
    text-decoration: underline;
  }

  strong {
    font-weight: 600;
  }

  img {
    max-width: 100%;
    border-radius: calc(var(--radius) - 6px);
  }
</style>
