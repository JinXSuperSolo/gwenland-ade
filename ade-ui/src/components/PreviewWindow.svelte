<script lang="ts">
  import { onMount } from "svelte";
  import PreviewPane from "./PreviewPane.svelte";
  import { initConversationListeners } from "../features/chat/conversation.svelte";

  // The detached preview is its own webview (separate JS runtime from the main
  // window): it must subscribe to the same ade:// stream events so output keeps
  // flowing while popped out. Reattach is driven from the Rust side — when this
  // window is destroyed it emits `ade://preview-closed` to the main window,
  // which flips `previewDetached` back off (see lib.rs / App.svelte).
  onMount(() => initConversationListeners());
</script>

<div class="detached-root" data-tauri-drag-region>
  <PreviewPane detachedView />
</div>

<style>
  .detached-root {
    height: 100vh;
    width: 100vw;
    background: var(--background);
    overflow: hidden;
  }
</style>
