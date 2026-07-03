<script lang="ts">
  import type { Component } from "svelte";
  import type { IconComponentProps } from "phosphor-svelte";
  import PlusIcon from "phosphor-svelte/lib/PlusIcon";
  import LightningIcon from "phosphor-svelte/lib/LightningIcon";
  import FolderSimpleIcon from "phosphor-svelte/lib/FolderSimpleIcon";
  import PaletteIcon from "phosphor-svelte/lib/PaletteIcon";
  import { ui } from "./ui.svelte";

  let hover = $state(false);
  const visible = $derived(hover || ui.sidebarPinned);

  const workspaceItems: {
    label: string;
    icon: Component<IconComponentProps>;
  }[] = [
    { label: "New Chat", icon: PlusIcon },
    { label: "Projects", icon: FolderSimpleIcon },
    { label: "Skills", icon: LightningIcon },
    { label: "Design", icon: PaletteIcon },
  ];

  const historyItems = [
    "Tauri Dev Server Issue",
    "Sidebar UI Redesign",
    "Implement History Section",
  ];
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -- mouse-only hover reveal zone -->
<div
  class="trigger"
  class:pinned={ui.sidebarPinned}
  onmouseenter={() => (hover = true)}
  onmouseleave={() => (hover = false)}
>
  <nav class="sidebar" class:visible>
    <div class="menu-items">
      <div class="section-label">Workspace</div>
      {#each workspaceItems as item}
        {@const Icon = item.icon}
        <button class="item">
          <span class="icon"><Icon size={14} /></span>
          <span class="label">{item.label}</span>
        </button>
      {/each}
    </div>

    <div class="menu-items" style="margin-top: 12px;">
      <div class="section-label">History</div>
      {#each historyItems as item}
        <button class="item history-item">
          <span class="label">{item}</span>
        </button>
      {/each}
    </div>

    <div class="footer">
      <span class="version">v0.1.0</span>
    </div>
  </nav>
</div>

<style>
  .trigger {
    position: fixed;
    left: 0;
    top: 40px;
    bottom: 0;
    width: 12px;
    z-index: 50;
    pointer-events: auto;
  }

  /* widen the hover zone once revealed or pinned so it doesn't flicker shut */
  .trigger:hover,
  .trigger.pinned {
    width: 220px;
  }

  .sidebar {
    position: absolute;
    left: 12px;
    top: 12px;
    bottom: 12px;
    width: 220px;
    background: var(--sidebar);
    border: 1px solid var(--sidebar-border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-2xl);
    padding: 8px 6px;
    display: flex;
    flex-direction: column;
    transform: translateX(calc(-100% - 12px));
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .menu-items {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .section-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted-foreground);
    padding: 6px 8px 4px 8px;
    opacity: 0.7;
    user-select: none;
  }

  .footer {
    margin-top: auto;
    padding: 6px 8px;
  }

  .version {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--muted-foreground);
    opacity: 0.6;
  }

  .sidebar.visible {
    transform: translateX(0);
  }

  .item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--muted-foreground);
    font-family: var(--font-sans);
    font-size: 12px;
    cursor: pointer;
    width: 100%;
    text-align: left;
    transition:
      background 0.15s,
      color 0.15s;
  }

  .item:hover {
    background: var(--sidebar-accent);
    color: var(--sidebar-accent-foreground);
  }

  .history-item {
    font-size: 13px;
    padding-left: 8px;
  }

  .history-item .label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
  }

  .icon {
    width: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
</style>
