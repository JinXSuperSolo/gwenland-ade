<script lang="ts">
  import { onMount } from "svelte";
  import Composer from "../features/chat/Composer.svelte";
  import PreviewPane from "./PreviewPane.svelte";
  import { ui } from "../shared/ui.svelte";

  // Min widths from the spec (GWEN-489).
  const MIN_COMPOSER = 320;
  const MIN_PREVIEW = 400;
  const STORAGE_KEY = "ade.splitRatio";

  // Fraction of total width allotted to the composer (left) pane.
  let ratio = $state(loadRatio());
  let container = $state<HTMLDivElement>();
  let dragging = $state(false);

  function loadRatio(): number {
    const raw = Number(localStorage.getItem(STORAGE_KEY));
    return raw > 0 && raw < 1 ? raw : 0.5;
  }

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging || !container) return;
    const rect = container.getBoundingClientRect();
    let x = e.clientX - rect.left;
    // Clamp so neither pane drops below its min width.
    x = Math.max(MIN_COMPOSER, Math.min(x, rect.width - MIN_PREVIEW));
    ratio = x / rect.width;
  }

  function onPointerUp(e: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    localStorage.setItem(STORAGE_KEY, String(ratio));
  }

  // The right pane is shown only when the preview is visible AND not detached.
  // Otherwise the composer takes the full width.
  let showPreview = $derived(ui.previewVisible && !ui.previewDetached);
  let columns = $derived(
    showPreview
      ? `minmax(${MIN_COMPOSER}px, ${ratio}fr) 6px minmax(${MIN_PREVIEW}px, ${1 - ratio}fr)`
      : "1fr",
  );

  onMount(() => {
    // Keep the ratio valid if the window is resized very small.
    const onResize = () => {
      if (!container) return;
      const w = container.getBoundingClientRect().width;
      const minR = MIN_COMPOSER / w;
      const maxR = 1 - MIN_PREVIEW / w;
      if (minR < maxR) ratio = Math.max(minR, Math.min(ratio, maxR));
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });
</script>

<div class="split" bind:this={container} style="grid-template-columns: {columns};">
  <div class="pane composer-pane">
    <Composer />
  </div>

  {#if showPreview}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="handle"
      class:dragging
      onpointerdown={onPointerDown}
      onpointermove={onPointerMove}
      onpointerup={onPointerUp}
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize panes"
    >
      <span class="grip"></span>
    </div>

    <div class="pane preview-pane">
      <PreviewPane />
    </div>
  {/if}
</div>

<style>
  .split {
    display: grid;
    height: 100%;
    width: 100%;
    overflow: hidden;
  }

  .pane {
    min-width: 0;
    height: 100%;
    overflow: hidden;
  }

  .composer-pane {
    background: var(--background);
  }

  /* Preview sits on a faintly elevated surface for pane separation without a
     border (design-system: no outlines). */
  .preview-pane {
    background: color-mix(in srgb, var(--card) 55%, var(--background));
    border-radius: var(--radius) 0 0 0;
    box-shadow: inset 2px 0 6px -4px var(--shadow-color, #000);
  }

  .handle {
    position: relative;
    cursor: col-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    touch-action: none;
  }

  .grip {
    width: 2px;
    height: 42px;
    border-radius: 2px;
    background: var(--muted-foreground);
    opacity: 0.15;
    transition: opacity 0.15s, background 0.15s;
  }

  .handle:hover .grip,
  .handle.dragging .grip {
    opacity: 0.6;
    background: var(--primary);
  }
</style>
