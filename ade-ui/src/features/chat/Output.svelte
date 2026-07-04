<script lang="ts">
  import SparkleIcon from 'phosphor-svelte/lib/SparkleIcon';
  import Feedback from '../../components/Feedback.svelte';
  import Markdown from '../renderers/Markdown.svelte';
  import ToolCall from './ToolCall.svelte';
  import AskPrompt from './AskPrompt.svelte';
  import type { Message } from './conversation.svelte';

  let {
    messages,
    isStreaming = false,
  }: { messages: Message[]; isStreaming?: boolean } = $props();

  let lastIndex = $derived(messages.length - 1);
</script>

<div class="output">
  {#each messages as msg, i}
    {#if msg.role === 'ade'}
      <div class="msg ade">
        <span class="mark"><SparkleIcon size={15} weight="fill" /></span>
        <div class="body">
          {#if msg.blocks && msg.blocks.length}
            {#each msg.blocks as block}
              {#if block.kind === 'text'}
                {#if block.text.trim()}
                  <div class="content md"><Markdown source={block.text} /></div>
                {/if}
              {:else if block.kind === 'tool'}
                <ToolCall {block} />
              {:else if block.kind === 'ask'}
                <AskPrompt {block} />
              {/if}
            {/each}
          {:else}
            <div class="content md"><Markdown source={msg.content} /></div>
          {/if}

          {#if i === lastIndex && !isStreaming && msg.content.trim()}
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
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 24px;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 80px 20px 24px;
    height: 100%;
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

  .content.md {
    white-space: normal;
  }
</style>
