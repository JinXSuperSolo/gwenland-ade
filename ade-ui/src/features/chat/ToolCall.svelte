<script lang="ts">
  import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
  import CircleNotchIcon from "phosphor-svelte/lib/CircleNotchIcon";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import WarningIcon from "phosphor-svelte/lib/WarningIcon";
  import TerminalIcon from "phosphor-svelte/lib/TerminalIcon";
  import type { ToolBlock } from "./conversation.svelte";

  let { block }: { block: ToolBlock } = $props();
  let expanded = $state(false);

  // A short, human label for the tool + its salient argument.
  const label = $derived(prettyLabel(block));

  function prettyLabel(b: ToolBlock): string {
    const a = (b.input ?? {}) as Record<string, unknown>;
    const arg =
      (a.path as string) ??
      (a.pattern as string) ??
      (a.command as string) ??
      (a.url as string) ??
      "";
    const name = b.name.replace(/^GL_/, "").replace(/_/g, " ");
    return arg ? `${name} · ${arg}` : name;
  }
</script>

<div class="tool" class:error={block.status === "error"}>
  <button class="head" onclick={() => (expanded = !expanded)}>
    <span class="chev" class:open={expanded}><CaretRightIcon size={12} weight="bold" /></span>
    <span class="ico">
      {#if block.status === "running"}
        <span class="spin"><CircleNotchIcon size={13} weight="bold" /></span>
      {:else if block.status === "error"}
        <WarningIcon size={13} weight="fill" />
      {:else}
        <TerminalIcon size={13} weight="bold" />
      {/if}
    </span>
    <span class="label">{label}</span>
    {#if block.status === "done"}<span class="tick"><CheckIcon size={12} weight="bold" /></span>{/if}
  </button>

  {#if expanded}
    <div class="detail">
      <div class="section-label">Input</div>
      <pre class="code">{JSON.stringify(block.input, null, 2)}</pre>
      {#if block.result != null}
        <div class="section-label">Result</div>
        <pre class="code">{block.result}</pre>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tool {
    background: var(--secondary);
    border-radius: calc(var(--radius) - 4px);
    overflow: hidden;
    font-family: var(--font-sans);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 100%;
    padding: 7px 10px;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    color: var(--muted-foreground);
    transition: color 0.15s;
  }
  .head:hover { color: var(--foreground); }

  .chev {
    display: flex;
    transition: transform 0.15s;
    flex-shrink: 0;
  }
  .chev.open { transform: rotate(90deg); }

  .ico { display: flex; color: var(--primary); flex-shrink: 0; }
  .error .ico { color: var(--destructive); }

  .spin { display: flex; animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .label {
    flex: 1;
    font-size: 12.5px;
    font-family: var(--font-mono);
    color: var(--foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tick { display: flex; color: var(--primary); flex-shrink: 0; }

  .detail {
    padding: 0 10px 10px 29px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .section-label {
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted-foreground);
    opacity: 0.7;
    margin-top: 6px;
  }

  .code {
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 1.5;
    color: var(--foreground);
    background: var(--background);
    border-radius: calc(var(--radius) - 8px);
    padding: 8px 10px;
    overflow-x: auto;
    max-height: 240px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
