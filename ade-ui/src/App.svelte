<script lang="ts">
  import Titlebar from './Titlebar.svelte';
  import Sidebar from './Sidebar.svelte';
  import Composer from './Composer.svelte';
  import Settings from './Settings.svelte';
  import { ui } from './ui.svelte';
</script>

<div class="app">
  <Titlebar />
  <Sidebar />
  <main>
    <Composer />
  </main>
  {#if ui.settingsOpen}
    <Settings onClose={() => (ui.settingsOpen = false)} />
  {/if}
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
