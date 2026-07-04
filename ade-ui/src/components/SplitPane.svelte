<script lang="ts">
  import { onMount } from "svelte";
  import ChatView from "../features/chat/ChatView.svelte";
  import PreviewPane from "./PreviewPane.svelte";
  import { ui } from "../shared/ui.svelte";

  // Min widths: the chat stays the primary surface; the artifact preview opens
  // beside it (GWEN-489, revised for the artifact-preview model).
  const MIN_COMPOSER = 360;
  const MIN_PREVIEW = 380;
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

  // The grid keeps three tracks at all times so opening/closing the preview
  // animates smoothly (a track-count change can't transition). When hidden, the
  // handle + preview tracks collapse to 0 and the transition on the grid width
  // slides the chat to full width.
  let columns = $derived(
    showPreview
      ? `minmax(${MIN_COMPOSER}px, ${ratio}fr) 6px minmax(${MIN_PREVIEW}px, ${1 - ratio}fr)`
      : `1fr 0px 0px`,
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

<div class="split" class:preview-open={showPreview} bind:this={container} style="grid-template-columns: {columns};">
  <div class="pane composer-pane">
    <ChatView />
  </div>

  <!-- Handle + preview stay mounted so the open/close animates; they collapse
       to zero width and become non-interactive when hidden. -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="handle"
    class:dragging
    onpointerdown={showPreview ? onPointerDown : undefined}
    onpointermove={showPreview ? onPointerMove : undefined}
    onpointerup={showPreview ? onPointerUp : undefined}
    role="separator"
    aria-orientation="vertical"
    aria-label="Resize panes"
    aria-hidden={!showPreview}
  >
    <span class="grip"></span>
  </div>

  <div class="pane preview-pane" aria-hidden={!showPreview}>
    <PreviewPane />
  </div>
</div>

<style>
  .split {
    display: grid;
    height: 100%;
    width: 100%;
    overflow: hidden;
    /* Animate the column tracks so the preview slides open/closed. Skipped
       mid-drag (see .dragging) so resizing stays 1:1 with the pointer. */
    transition: grid-template-columns 0.32s cubic-bezier(0.4, 0, 0.2, 1);
  }

  /* No column animation while actively dragging the handle. */
  .split:has(.handle.dragging) {
    transition: none;
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
    /* Slide + fade the content in from the right as the track opens. */
    opacity: 0;
    transform: translateX(16px);
    pointer-events: none;
    transition: opacity 0.28s ease, transform 0.32s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .split.preview-open .preview-pane {
    opacity: 1;
    transform: translateX(0);
    pointer-events: auto;
  }

  .handle {
    position: relative;
    cursor: col-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    touch-action: none;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.28s ease;
  }

  .split.preview-open .handle {
    opacity: 1;
    pointer-events: auto;
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
