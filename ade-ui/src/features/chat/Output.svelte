<script lang="ts">
  import SparkleIcon from 'phosphor-svelte/lib/SparkleIcon';
  import Feedback from '../../components/Feedback.svelte';
  import Markdown from '../renderers/Markdown.svelte';

  type Message = { role: 'user' | 'ade'; content: string; prompt?: string };
  let {
    messages,
    isStreaming = false,
  }: { messages: Message[]; isStreaming?: boolean } = $props();

  // Feedback attaches only to the last ADE message, once it's finished streaming.
  let lastIndex = $derived(messages.length - 1);
</script>

<div class="output">
  {#each messages as msg, i}
    {#if msg.role === 'ade'}
      <div class="msg ade">
        <span class="mark"><SparkleIcon size={15} weight="fill" /></span>
        <div class="body">
          <div class="content md"><Markdown source={msg.content} /></div>
          {#if i === lastIndex && !isStreaming}
            <Feedback prompt={msg.prompt ?? ''} output={msg.content} />
          {/if}
        </div>
      </div>
    {:else}
      <div class="msg user">
        <p class="content">{msg.content}</p>
      </div>
    {/if}
  {/each}
</div>

<style>
  .output {
    width: 100%;
    max-width: 680px;
    display: flex;
    flex-direction: column;
    gap: 24px;
    overflow-y: auto;
    padding: 80px 0 24px;
    flex: 1;
  }

  .msg.user {
    align-self: flex-end;
    width: fit-content;
    max-width: 100%;
    background: var(--card);
    border-radius: var(--radius);
    padding: 10px 14px;
    box-shadow: var(--shadow-xs);
  }

  .msg.user .content {
    color: var(--card-foreground);
    font-size: 14px;
    white-space: pre-wrap;
  }

  .msg.ade {
    display: flex;
    gap: 10px;
    padding-right: 8px;
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
    flex: 1;
  }

  .mark {
    display: flex;
    flex-shrink: 0;
    margin-top: 4px;
    color: var(--primary);
  }

  .content {
    font-family: var(--font-sans);
    font-size: 15px;
    letter-spacing: var(--tracking-normal);
    color: var(--foreground);
    line-height: 1.7;
  }

  /* ADE markdown output — block flow, not pre-wrapped. */
  .content.md {
    white-space: normal;
  }
</style>
