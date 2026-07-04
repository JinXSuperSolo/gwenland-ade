<script lang="ts">
  import Composer from "./Composer.svelte";
  import ConversationView from "./ConversationView.svelte";
  import { isActive } from "./conversation.svelte";

  // The main chat surface (Claude.ai-style): once a conversation starts, the
  // thread fills the top and the composer sits at the bottom. Before that, the
  // composer is centered on its own (idle greeting lives inside Composer).
  let active = $derived(isActive());
</script>

<div class="chat" class:active>
  {#if active}
    <div class="thread">
      <ConversationView />
    </div>
  {/if}
  <div class="composer-dock" class:floating={active}>
    <Composer />
  </div>
</div>

<style>
  .chat {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
    overflow: hidden;
  }

  .thread {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Idle: composer fills and centers itself. Active: it docks at the bottom. */
  .composer-dock {
    flex-shrink: 0;
  }
  .chat:not(.active) .composer-dock {
    flex: 1;
    min-height: 0;
  }
</style>
