<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Titlebar from "./Titlebar.svelte";
  import Sidebar from "./Sidebar.svelte";
  import SplitPane from "./SplitPane.svelte";
  import StatusBar from "./StatusBar.svelte";
  import Settings from "./Settings.svelte";
  import { ui, openSettings } from "./ui.svelte";
  import { chat, initConversationListeners } from "./conversation.svelte";
  import { onboarding } from "./onboarding.svelte";

  // Auto-reveal the preview pane the moment there's output to show (or a tool
  // starts producing one). Hidden by default; the titlebar toggle still lets
  // the user hide/show it manually afterwards.
  $effect(() => {
    if (chat.messages.length > 0 && !ui.previewVisible && !ui.previewDetached) {
      ui.previewVisible = true;
    }
  });

  onMount(() => {
    const cleanup = initConversationListeners();

    // First-time detection (GWEN-490): no memory yet → onboarding.
    invoke<boolean>("has_memory")
      .then((has) => (onboarding.isFirstTime = !has))
      .catch(() => {});

    // Reattach: the Rust side emits this when the detached preview window is
    // destroyed, so the split layout can come back.
    const unlistenReattach = listen("ade://preview-closed", () => {
      ui.previewDetached = false;
    });

    // Global Ctrl+M opens Settings on the Memory section (GWEN-491).
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "m") {
        e.preventDefault();
        openSettings("memory");
      }
    };
    window.addEventListener("keydown", onKey);

    return () => {
      cleanup();
      unlistenReattach.then((fn) => fn());
      window.removeEventListener("keydown", onKey);
    };
  });
</script>

<div class="app">
  <Titlebar />
  <Sidebar />
  <main>
    {#if ui.settingsOpen}
      <Settings onClose={() => { ui.settingsOpen = false; ui.settingsSection = null; }} />
    {:else}
      <SplitPane />
    {/if}
  </main>
  <StatusBar />
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--background);
    overflow: hidden;
  }

  main {
    flex: 1;
    overflow: hidden;
    position: relative;
  }
</style>
