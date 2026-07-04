<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import PencilSimpleIcon from "phosphor-svelte/lib/PencilSimpleIcon";
  import CopyIcon from "phosphor-svelte/lib/CopyIcon";
  import PlayIcon from "phosphor-svelte/lib/PlayIcon";
  import ThumbsUpIcon from "phosphor-svelte/lib/ThumbsUpIcon";
  import ThumbsDownIcon from "phosphor-svelte/lib/ThumbsDownIcon";
  import ArrowClockwiseIcon from "phosphor-svelte/lib/ArrowClockwiseIcon";

  // The prompt that produced `output`, needed for reflection context (GWEN-485).
  let { prompt, output }: { prompt: string; output: string } = $props();

  type Phase = "idle" | "tweaking" | "done";
  let phase = $state<Phase>("idle");
  let verdict = $state<"accept" | "reject" | "tweak" | null>(null);
  let tweakText = $state("");

  async function send(action: "accept" | "reject" | "tweak", tweak?: string) {
    try {
      await invoke("record_feedback", {
        request: { action, prompt, output, tweak: tweak ?? null },
      });
    } catch (err) {
      // Feedback is best-effort; never block the UI on it.
      console.error("record_feedback failed:", err);
    }
    verdict = action;
    phase = "done";
  }

  function submitTweak() {
    const t = tweakText.trim();
    if (!t) return;
    send("tweak", t);
  }

  function cancelTweak() {
    phase = "idle";
    tweakText = "";
  }
</script>

{#if phase === "done"}
  <div class="feedback done" role="status">
    {#if verdict === "accept"}
      <CheckIcon size={13} weight="bold" /> Thanks — noted.
    {:else if verdict === "reject"}
      <XIcon size={13} weight="bold" /> Recorded as a miss.
    {:else}
      <PencilSimpleIcon size={13} /> Preference saved.
    {/if}
  </div>
{:else if phase === "tweaking"}
  <div class="tweak">
    <textarea
      bind:value={tweakText}
      placeholder="How should ADE have responded?"
      rows="2"
    ></textarea>
    <div class="tweak-actions">
      <button class="btn-text" onclick={cancelTweak}>Cancel</button>
      <button class="btn-primary" onclick={submitTweak} disabled={!tweakText.trim()}>
        Save preference
      </button>
    </div>
  </div>
{:else}
  <div class="feedback">
    <button class="fb" aria-label="Copy" title="Copy">
      <CopyIcon size={14} />
    </button>
    <button class="fb" aria-label="Play" title="Read Aloud">
      <PlayIcon size={14} />
    </button>
    <button class="fb" aria-label="Good Response" title="Good Response">
      <ThumbsUpIcon size={14} />
    </button>
    <button class="fb" aria-label="Bad Response" title="Bad Response">
      <ThumbsDownIcon size={14} />
    </button>
    <button class="fb" aria-label="Regenerate" title="Regenerate">
      <ArrowClockwiseIcon size={14} />
    </button>
  </div>
{/if}

<style>
  .feedback {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .feedback.done {
    gap: 6px;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--muted-foreground);
  }

  .fb {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 6px);
    color: var(--muted-foreground);
    cursor: pointer;
    transition:
      background 0.15s,
      color 0.15s;
  }

  .fb:hover {
    background: var(--secondary);
    color: var(--foreground);
  }

  .tweak {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    background: var(--card);
    border-radius: var(--radius);
    padding: 10px 12px;
    box-shadow: var(--shadow-xs);
  }

  .tweak textarea {
    width: 100%;
    background: transparent;
    border: none;
    outline: none;
    resize: none;
    font-family: var(--font-sans);
    font-size: 14px;
    color: var(--foreground);
    line-height: 1.6;
  }

  .tweak textarea::placeholder {
    color: var(--muted-foreground);
  }

  .tweak-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .btn-text {
    background: transparent;
    border: none;
    color: var(--muted-foreground);
    font-family: var(--font-sans);
    font-size: 12.5px;
    padding: 6px 10px;
    border-radius: calc(var(--radius) - 6px);
    cursor: pointer;
    transition: color 0.15s;
  }

  .btn-text:hover {
    color: var(--foreground);
  }

  .btn-primary {
    background: var(--primary);
    border: none;
    color: var(--primary-foreground);
    font-family: var(--font-sans);
    font-size: 12.5px;
    padding: 6px 12px;
    border-radius: calc(var(--radius) - 6px);
    cursor: pointer;
    transition:
      background 0.15s,
      opacity 0.15s;
  }

  .btn-primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--primary) 85%, white);
  }

  .btn-primary:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
