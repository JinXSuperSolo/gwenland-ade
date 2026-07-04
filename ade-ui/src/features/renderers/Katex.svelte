<script lang="ts">
  import { renderMath } from "./katex";

  let { source, display = false }: { source: string; display?: boolean } = $props();

  const html = $derived(renderMath(source));
</script>

{#if display}
  <div class="math display">{@html html}</div>
{:else}
  <span class="math inline">{@html html}</span>
{/if}

<style>
  .math {
    font-family: "Cambria Math", "Latin Modern Math", var(--font-serif), serif;
    color: var(--foreground);
    line-height: normal;
  }

  .math.inline {
    padding: 0 1px;
    white-space: nowrap;
  }

  .math.display {
    display: block;
    text-align: center;
    margin: 0.9em 0;
    font-size: 1.15em;
    overflow-x: auto;
  }

  .math :global(i) {
    font-style: italic;
  }

  /* Fractions */
  .math :global(.k-frac) {
    display: inline-flex;
    flex-direction: column;
    vertical-align: middle;
    text-align: center;
    margin: 0 2px;
  }
  .math :global(.k-num) {
    padding: 0 4px;
    box-shadow: inset 0 -1px 0 currentColor;
  }
  .math :global(.k-den) {
    padding: 0 4px;
  }

  /* Sub/superscripts */
  .math :global(sup),
  .math :global(sub) {
    font-size: 0.72em;
    line-height: 0;
  }
  .math :global(.k-scripts) {
    display: inline-flex;
    flex-direction: column;
    vertical-align: middle;
    line-height: 1;
  }
  .math :global(.k-scripts sup),
  .math :global(.k-scripts sub) {
    line-height: 1;
  }

  /* Square root */
  .math :global(.k-sqrt) {
    display: inline-flex;
    align-items: flex-start;
  }
  .math :global(.k-radical) {
    margin-right: 1px;
  }
  .math :global(.k-under) {
    box-shadow: inset 0 1px 0 currentColor;
    padding: 0 2px;
  }
  .math :global(.k-root-idx) {
    font-size: 0.6em;
    align-self: flex-start;
    margin-right: -3px;
  }

  /* Accents */
  .math :global(.k-acc) {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    vertical-align: middle;
  }
  .math :global(.k-acc-mark) {
    line-height: 0.5;
    font-size: 0.9em;
  }

  .math :global(.k-op),
  .math :global(.k-text) {
    font-style: normal;
    font-family: var(--font-sans);
    padding: 0 1px;
  }
  .math :global(.k-op) {
    margin-right: 2px;
  }

  .math :global(.k-sym) {
    padding: 0 1px;
  }

  .math :global(.k-boxed) {
    padding: 2px 5px;
    box-shadow: 0 0 0 1px currentColor;
    border-radius: 3px;
  }

  .math :global(.k-quad) { display: inline-block; width: 1em; }
  .math :global(.k-thin) { display: inline-block; width: 0.22em; }
  .math :global(.k-sp) { display: inline-block; width: 0.6em; }

  .math :global(.k-err) {
    font-family: var(--font-mono);
    color: var(--destructive);
    font-size: 0.9em;
  }
</style>
