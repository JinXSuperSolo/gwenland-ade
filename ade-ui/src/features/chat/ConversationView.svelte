<script lang="ts">
  import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
  import Output from "./Output.svelte";
  import { chat, isActive } from "./conversation.svelte";

  // Standalone conversation thread: owns the centered scroll column, the empty
  // state, and the Output render. Both the in-split preview and the detached
  // window embed this, so the layout lives in exactly one place.
  //
  // `emptyText` lets the host customize the placeholder (e.g. "open in another
  // window" when detached).
  let {
    emptyText = "Output appears here once you describe something.",
  }: { emptyText?: string } = $props();

  let active = $derived(isActive());
</script>

<div class="conversation">
  {#if active}
    <Output messages={chat.messages} isStreaming={chat.isStreaming} />
  {:else}
    <div class="empty">
      <span class="mark"><SparkleIcon size={22} weight="fill" /></span>
      <p>{emptyText}</p>
    </div>
  {/if}
</div>

<style>
  /* Full-height flex column; the child (Output or empty state) fills it and
     handles its own centering + scroll. */
  .conversation {
    flex: 1;
    min-height: 0;
    width: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: fade-in 0.3s ease;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
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
