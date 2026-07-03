<script lang="ts">
  import SparkleIcon from 'phosphor-svelte/lib/SparkleIcon';

  type Message = { role: 'user' | 'ade'; content: string };
  let { messages }: { messages: Message[] } = $props();
</script>

<div class="output">
  {#each messages as msg}
    {#if msg.role === 'ade'}
      <div class="msg ade">
        <span class="mark"><SparkleIcon size={15} weight="fill" /></span>
        <p class="content">{msg.content}</p>
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
    padding: 24px 0;
    flex: 1;
  }

  .msg.user {
    align-self: flex-start;
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
  }

  .msg.ade {
    display: flex;
    gap: 10px;
    padding-right: 8px;
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
    white-space: pre-wrap;
  }
</style>
