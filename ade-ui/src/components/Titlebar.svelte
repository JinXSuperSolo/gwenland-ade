<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { platform } from '@tauri-apps/plugin-os';
  import SidebarSimpleIcon from 'phosphor-svelte/lib/SidebarSimpleIcon';
  import LayoutIcon from 'phosphor-svelte/lib/LayoutIcon';
  import GearSixIcon from 'phosphor-svelte/lib/GearSixIcon';
  import MagnifyingGlassIcon from 'phosphor-svelte/lib/MagnifyingGlassIcon';
  import ArrowLeftIcon from 'phosphor-svelte/lib/ArrowLeftIcon';
  import ArrowRightIcon from 'phosphor-svelte/lib/ArrowRightIcon';
  import MinusIcon from 'phosphor-svelte/lib/MinusIcon';
  import SquareIcon from 'phosphor-svelte/lib/SquareIcon';
  import XIcon from 'phosphor-svelte/lib/XIcon';
  import { ui, openSettings } from '../shared/ui.svelte';

  const win = getCurrentWindow();
  const os = platform();
  const isMac = os === 'macos';

  const minimize = () => win.minimize();
  const maximize = () => win.toggleMaximize();
  const close = () => win.close();
</script>

<div class="titlebar" data-tauri-drag-region>
  {#if isMac}
    <div class="controls mac">
      <button class="dot close" onclick={close} aria-label="Close"></button>
      <button class="dot minimize" onclick={minimize} aria-label="Minimize"></button>
      <button class="dot maximize" onclick={maximize} aria-label="Maximize"></button>
    </div>
  {/if}

  <div class="nav" class:mac-offset={isMac}>
    <button
      class="nav-btn"
      class:active={ui.sidebarPinned}
      onclick={() => (ui.sidebarPinned = !ui.sidebarPinned)}
      aria-label="Toggle sidebar"
    >
      <SidebarSimpleIcon size={16} />
    </button>
    <button class="nav-btn" aria-label="Search"><MagnifyingGlassIcon size={15} /></button>
    <button class="nav-btn" aria-label="Back"><ArrowLeftIcon size={15} /></button>
    <button class="nav-btn" aria-label="Forward"><ArrowRightIcon size={15} /></button>
    <button
      class="nav-btn"
      class:active={ui.settingsOpen}
      onclick={() => openSettings()}
      aria-label="Settings"
    >
      <GearSixIcon size={15} />
    </button>
  </div>

  <!-- Right-aligned actions: preview toggle sits by the window controls. -->
  <div class="actions" class:win-offset={!isMac}>
    <button
      class="nav-btn"
      class:active={ui.previewVisible}
      onclick={() => (ui.previewVisible = !ui.previewVisible)}
      aria-label="Toggle preview pane"
      title="Toggle preview"
    >
      <LayoutIcon size={15} />
    </button>
  </div>

  {#if !isMac}
    <div class="controls win">
      <button class="cap" onclick={minimize} aria-label="Minimize"><MinusIcon size={13} /></button>
      <button class="cap" onclick={maximize} aria-label="Maximize"><SquareIcon size={11} /></button>
      <button class="cap danger" onclick={close} aria-label="Close"><XIcon size={13} /></button>
    </div>
  {/if}
</div>

<style>
  .titlebar {
    height: 40px;
    display: flex;
    align-items: center;
    background: var(--background);
    position: relative;
    flex-shrink: 0;
  }

  .nav {
    position: absolute;
    left: 8px;
    display: flex;
    gap: 2px;
  }

  .nav.mac-offset {
    left: 84px;
  }

  /* Right-aligned action group (preview toggle). On Windows it clears the
     three 46px caption buttons; on macOS it sits flush right. */
  .actions {
    position: absolute;
    right: 8px;
    display: flex;
    gap: 2px;
  }

  .actions.win-offset {
    right: 146px;
  }

  .nav-btn {
    width: 32px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 8px);
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .nav-btn:hover {
    background: var(--secondary);
    color: var(--foreground);
  }

  .nav-btn.active {
    color: var(--primary);
  }

  /* macOS traffic lights */
  .controls.mac {
    position: absolute;
    left: 12px;
    display: flex;
    gap: 6px;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: none;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .dot:hover { opacity: 0.75; }
  .dot.close    { background: #ff5f57; }
  .dot.minimize { background: #ffbd2e; }
  .dot.maximize { background: #28c840; }

  /* Windows/Linux caption buttons */
  .controls.win {
    position: absolute;
    right: 0;
    display: flex;
  }

  .cap {
    width: 46px;
    height: 40px;
    border: none;
    background: transparent;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cap:hover { background: var(--secondary); color: var(--foreground); }
  .cap.danger:hover { background: #c42b1c; color: white; }
</style>
