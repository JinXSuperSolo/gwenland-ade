<script lang="ts">
  import QuestionIcon from "phosphor-svelte/lib/QuestionIcon";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import { answerAsk, type AskBlock } from "./conversation.svelte";

  let { block }: { block: AskBlock } = $props();

  let text = $state("");
  const answered = $derived(block.answer != null);

  function choose(value: string) {
    if (answered) return;
    answerAsk(block, value);
  }

  function submitText() {
    const v = text.trim();
    if (v) choose(v);
  }
</script>

<div class="ask" class:answered>
  <div class="q">
    <span class="ico"><QuestionIcon size={15} weight="fill" /></span>
    <span class="text">{block.question}</span>
  </div>

  {#if answered}
    <div class="chosen"><CheckIcon size={13} weight="bold" /> {block.answer}</div>
  {:else if block.options && block.options.length}
    <div class="options">
      {#each block.options as opt}
        <button class="opt" onclick={() => choose(opt)}>{opt}</button>
      {/each}
    </div>
  {:else}
    <div class="freeform">
      <input
        bind:value={text}
        onkeydown={(e) => e.key === "Enter" && submitText()}
        placeholder="Type your answer…"
      />
      <button class="send" onclick={submitText} disabled={!text.trim()}>Reply</button>
    </div>
  {/if}
</div>

<style>
  .ask {
    background: color-mix(in srgb, var(--primary) 7%, var(--card));
    border-radius: calc(var(--radius) - 2px);
    padding: 12px 14px;
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--primary) 22%, transparent);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .q {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }
  .ico { display: flex; color: var(--primary); margin-top: 2px; flex-shrink: 0; }
  .text {
    font-family: var(--font-sans);
    font-size: 14px;
    color: var(--foreground);
    line-height: 1.5;
  }

  .options {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .opt {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--foreground);
    background: var(--card);
    border: none;
    border-radius: calc(var(--radius) - 6px);
    padding: 8px 14px;
    cursor: pointer;
    box-shadow: var(--shadow-xs);
    transition: background 0.15s, color 0.15s;
  }
  .opt:hover {
    background: var(--primary);
    color: var(--primary-foreground);
  }

  .freeform {
    display: flex;
    gap: 8px;
  }
  input {
    flex: 1;
    background: var(--card);
    border: none;
    outline: none;
    border-radius: calc(var(--radius) - 6px);
    padding: 8px 12px;
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--foreground);
  }
  input:focus {
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 35%, transparent);
  }
  .send {
    background: var(--primary);
    border: none;
    color: var(--primary-foreground);
    font-family: var(--font-sans);
    font-size: 13px;
    padding: 8px 16px;
    border-radius: calc(var(--radius) - 6px);
    cursor: pointer;
  }
  .send:disabled { opacity: 0.4; cursor: default; }

  .chosen {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    align-self: flex-start;
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--primary);
  }
</style>
