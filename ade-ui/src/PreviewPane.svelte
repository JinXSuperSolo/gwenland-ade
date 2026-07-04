<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import CornersOutIcon from "phosphor-svelte/lib/CornersOutIcon";
  import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
  import Output from "./Output.svelte";
  import { chat, isActive } from "./conversation.svelte";
  import { ui } from "./ui.svelte";
  import { onboarding, maybeShowDetachHint, dismissDetachHint } from "./onboarding.svelte";

  // When true, the pane renders as the standalone detached window's whole body
  // (no detach button, no empty-state chrome). Default is the in-split pane.
  let { detachedView = false }: { detachedView?: boolean } = $props();

  let active = $derived(isActive());

  // Surface the one-time "detach preview" hint after the first generate
  // (GWEN-490). Only fires in the in-split pane, for first-time users.
  $effect(() => {
    if (!detachedView && chat.hasGeneratedOnce) maybeShowDetachHint();
  });

  async function detach() {
    try {
      await invoke("open_preview_window");
      ui.previewDetached = true;
    } catch (err) {
      console.error("detach failed", err);
    }
  }
</script>

<div class="preview" class:detached-view={detachedView}>
  {#if !detachedView}
    <div class="bar" data-tauri-drag-region>
      <span class="label">Preview</span>
      <div class="detach-wrap">
        {#if onboarding.showDetachHint}
          <span class="hint">Tip: pop the output out into a floating window →</span>
        {/if}
        <button class="detach" onclick={detach} aria-label="Detach preview into a floating window" title="Detach preview">
          <CornersOutIcon size={15} />
        </button>
      </div>
    </div>
  {/if}

  <div class="pane-body">
    {#if ui.previewDetached && !detachedView}
      <div class="empty">
        <span class="mark"><SparkleIcon size={22} weight="fill" /></span>
        <p>Preview is open in a separate window.</p>
      </div>
    {:else if active}
      <Output messages={chat.messages} isStreaming={chat.isStreaming} />
    {:else}
      <div class="empty">
        <span class="mark"><SparkleIcon size={22} weight="fill" /></span>
        <p>Output appears here once you describe something.</p>
      </div>
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
    padding: 0 10px 0 16px;
    flex-shrink: 0;
  }

  .label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    opacity: 0.7;
    user-select: none;
  }

  .detach-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .hint {
    font-family: var(--font-sans);
    font-size: 11px;
    color: var(--primary);
    background: color-mix(in srgb, var(--primary) 12%, transparent);
    padding: 3px 8px;
    border-radius: calc(var(--radius) - 8px);
    white-space: nowrap;
    animation: hint-in 0.25s ease;
  }

  @keyframes hint-in {
    from { opacity: 0; transform: translateX(6px); }
    to { opacity: 1; transform: translateX(0); }
  }

  .detach {
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

  .detach:hover {
    background: color-mix(in srgb, var(--primary) 15%, transparent);
    color: var(--primary);
  }

  .pane-body {
    flex: 1;
    min-height: 0;
    display: flex;
    justify-content: center;
    overflow: hidden;
    animation: fade-in 0.3s ease;
  }

  .detached-view .pane-body {
    padding-top: 12px;
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
