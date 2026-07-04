<script lang="ts">
  import { parseMarkdown, type Block, type Inline } from "./markdown";
  import InlineNodes from "./MarkdownInline.svelte";
  import Mermaid from "./Mermaid.svelte";
  import Katex from "./Katex.svelte";
  import Self from "./Markdown.svelte";

  // Either raw source (top-level) or a pre-parsed block list (recursion).
  let { source, blocks }: { source?: string; blocks?: Block[] } = $props();

  const tree = $derived<Block[]>(blocks ?? (source != null ? parseMarkdown(source) : []));
</script>

{#each tree as block}
  {#if block.type === "heading"}
    {#if block.level === 1}<h1><InlineNodes nodes={block.children} /></h1>
    {:else if block.level === 2}<h2><InlineNodes nodes={block.children} /></h2>
    {:else if block.level === 3}<h3><InlineNodes nodes={block.children} /></h3>
    {:else if block.level === 4}<h4><InlineNodes nodes={block.children} /></h4>
    {:else if block.level === 5}<h5><InlineNodes nodes={block.children} /></h5>
    {:else}<h6><InlineNodes nodes={block.children} /></h6>
    {/if}
  {:else if block.type === "paragraph"}
    <p><InlineNodes nodes={block.children} /></p>
  {:else if block.type === "code"}
    {#if block.lang === "mermaid"}
      <Mermaid source={block.value} />
    {:else}
      <pre class="code"><code>{block.value}</code></pre>
    {/if}
  {:else if block.type === "mathBlock"}
    <Katex source={block.value} display />
  {:else if block.type === "blockquote"}
    <blockquote><Self blocks={block.children} /></blockquote>
  {:else if block.type === "hr"}
    <hr />
  {:else if block.type === "list"}
    {#if block.ordered}
      <ol start={block.start}>
        {#each block.items as item}
          <li><Self blocks={item} /></li>
        {/each}
      </ol>
    {:else}
      <ul>
        {#each block.items as item}
          <li><Self blocks={item} /></li>
        {/each}
      </ul>
    {/if}
  {:else if block.type === "table"}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            {#each block.header as cell, c}
              <th style={alignStyle(block.align[c])}><InlineNodes nodes={cell} /></th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each block.rows as row}
            <tr>
              {#each row as cell, c}
                <td style={alignStyle(block.align[c])}><InlineNodes nodes={cell} /></td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
{/each}

<script module lang="ts">
  function alignStyle(a: "left" | "center" | "right" | null): string {
    return a ? `text-align:${a}` : "";
  }
  // Re-export types so consumers can pass either prop.
  export type { Block, Inline };
</script>

<style>
  /* Markdown block spacing — tuned to sit inside the ADE output thread. */
  :global(.md) > :first-child {
    margin-top: 0;
  }
  :global(.md) > :last-child {
    margin-bottom: 0;
  }

  h1, h2, h3, h4, h5, h6 {
    font-family: var(--font-sans);
    font-weight: 600;
    color: var(--foreground);
    line-height: 1.3;
    margin: 1.4em 0 0.5em;
  }
  h1 { font-size: 1.5em; }
  h2 { font-size: 1.3em; }
  h3 { font-size: 1.15em; }
  h4 { font-size: 1.02em; }
  h5, h6 { font-size: 0.95em; color: var(--muted-foreground); }

  p {
    margin: 0.6em 0;
    line-height: 1.7;
  }

  ul, ol {
    margin: 0.6em 0;
    padding-left: 1.5em;
  }
  li {
    margin: 0.25em 0;
    line-height: 1.6;
  }
  li > :global(p) {
    margin: 0.2em 0;
  }

  blockquote {
    margin: 0.7em 0;
    padding: 0.2em 0 0.2em 1em;
    color: var(--muted-foreground);
    box-shadow: inset 3px 0 0 color-mix(in srgb, var(--primary) 50%, transparent);
    border-radius: 2px;
  }

  hr {
    border: none;
    height: 1px;
    background: var(--border);
    margin: 1.2em 0;
  }

  pre.code {
    margin: 0.7em 0;
    padding: 12px 14px;
    background: var(--secondary);
    border-radius: calc(var(--radius) - 4px);
    overflow-x: auto;
  }
  pre.code code {
    font-family: var(--font-mono);
    font-size: 12.5px;
    line-height: 1.55;
    color: var(--foreground);
    white-space: pre;
  }

  .table-wrap {
    overflow-x: auto;
    margin: 0.8em 0;
  }
  table {
    border-collapse: collapse;
    width: 100%;
    font-size: 13px;
  }
  th, td {
    padding: 6px 12px;
    text-align: left;
    box-shadow: inset 0 -1px 0 var(--border);
  }
  th {
    font-weight: 600;
    color: var(--foreground);
  }
  td {
    color: var(--foreground);
  }
</style>
