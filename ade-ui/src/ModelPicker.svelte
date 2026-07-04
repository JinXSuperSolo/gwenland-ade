<script lang="ts">
  import { onMount } from "svelte";
  import ProviderIcon from "./ProviderIcon.svelte";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
  import InfoIcon from "phosphor-svelte/lib/InfoIcon";
  import { listProviders, type Provider, type Model } from "./providers";

  // Two-way bound selection: `{ providerId, modelId }`.
  let {
    providerId = $bindable(),
    modelId = $bindable(),
  }: { providerId: string; modelId: string } = $props();

  let providers = $state<Provider[]>([]);
  let open = $state(false);

  let effortLevel = $state("Low");
  const effortLevels = ["Low", "Medium", "High", "Max"];
  let effortMenuOpen = $state(false);
  let thinkingEnabled = $state(false);

  // Close effort menu when main picker closes
  $effect(() => {
    if (!open) effortMenuOpen = false;
  });

  onMount(async () => {
    providers = await listProviders();
    // Default to the first model of the first provider if nothing is set.
    if (!modelId && providers[0]?.models[0]) {
      providerId = providers[0].id;
      modelId = providers[0].models[0].id;
    }
  });

  const selected = $derived.by(() => {
    for (const p of providers) {
      const m = p.models.find((m) => m.id === modelId && p.id === providerId);
      if (m) return { provider: p, model: m };
    }
    return null;
  });

  function pick(p: Provider, m: Model) {
    providerId = p.id;
    modelId = m.id;
    open = false;
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && (open = false)} />

<div class="picker">
  <button class="trigger" onclick={() => (open = !open)}>
    {#if selected}
      <span class="tico"><ProviderIcon provider={selected.provider.id} size={14} /></span>
      <span class="tname">{selected.model.displayName}</span>
    {:else}
      <span class="tname muted">Select model</span>
    {/if}
    <svg class="caret" class:up={open} width="10" height="10" viewBox="0 0 24 24" fill="none">
      <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="backdrop" onclick={() => (open = false)}></div>
    <div class="menu" role="listbox">
      <div class="models-list">
        {#each providers as p (p.id)}
          {#each p.models as m (m.id)}
            {@const active = p.id === providerId && m.id === modelId}
            <button
              class="opt"
              class:active
              role="option"
              aria-selected={active}
              onclick={() => pick(p, m)}
            >
              <span class="opt-left">
                <span class="ico"><ProviderIcon provider={p.id} size={16} /></span>
                <span class="text">
                  <span class="name">{m.displayName}</span>
                  <span class="sub">{m.display || `${m.contextWindow/1000}K context window · $${m.inputPrice}/$${m.outputPrice} per 1M`}</span>
                </span>
              </span>
              {#if active}
                <span class="check"><CheckIcon size={16} weight="bold" /></span>
              {/if}
            </button>
          {/each}
        {/each}
      </div>
      
      <div class="menu-divider"></div>
      
      <div class="effort-wrapper">
        <button class="opt effort-opt" onclick={(e) => { e.stopPropagation(); effortMenuOpen = !effortMenuOpen; }}>
          <span class="name">Effort</span>
          <span class="effort-val">{effortLevel} <CaretRightIcon size={12} weight="bold" /></span>
        </button>

        {#if effortMenuOpen}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div class="effort-menu" onclick={(e) => e.stopPropagation()}>
            <p class="disclaimer">Higher effort means more thorough responses, but takes longer and uses your limits faster.</p>
            <div class="menu-divider" style="margin-bottom: 4px;"></div>
            
            <div class="effort-list">
              {#each effortLevels as level}
                <button class="effort-level-opt" onclick={() => { effortLevel = level; }}>
                  <span class="el-name">
                    {level}
                    {#if level === 'Low'}<span class="badge">Default</span>{/if}
                    {#if level === 'Max'}<InfoIcon size={12} class="info-ico" />{/if}
                  </span>
                  {#if effortLevel === level}
                    <span class="effort-check"><CheckIcon size={14} weight="bold" /></span>
                  {/if}
                </button>
              {/each}
            </div>

            <div class="menu-divider"></div>

            <div class="thinking-toggle">
              <div class="tt-text">
                <span class="tt-title">Thinking</span>
                <span class="tt-sub">Can think for more complex tasks</span>
              </div>
              <button class="switch" class:on={thinkingEnabled} aria-label="Toggle thinking" aria-pressed={thinkingEnabled} onclick={() => thinkingEnabled = !thinkingEnabled}>
                <div class="knob"></div>
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
  }

  .trigger {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 6px);
    color: var(--foreground);
    font-family: var(--font-sans);
    font-size: 12.5px;
    padding: 7px 9px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .trigger:hover {
    background: color-mix(in srgb, var(--primary) 15%, transparent);
  }

  .trigger:hover .tname,
  .trigger:hover .tico,
  .trigger:hover .caret {
    color: var(--primary);
  }

  .tico {
    display: flex;
    color: var(--foreground);
  }

  .tname.muted {
    color: var(--muted-foreground);
  }

  .caret {
    color: var(--muted-foreground);
    transition: transform 0.15s;
    margin-left: 1px;
  }

  .caret.up {
    transform: rotate(180deg);
  }

  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
  }

  /* Opens upward from the composer button */
  .menu {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    z-index: 61;
    width: 280px;
    background: var(--card);
    border-radius: calc(var(--radius) + 2px);
    box-shadow: var(--shadow-2xl);
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .models-list {
    max-height: 280px;
    overflow-y: auto;
    scrollbar-width: none;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .models-list::-webkit-scrollbar {
    display: none;
  }

  .opt {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px;
    border: none;
    background: transparent;
    border-radius: var(--radius);
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background 0.12s;
  }

  .opt-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .opt:hover {
    background: color-mix(in srgb, var(--primary) 15%, transparent);
  }

  .opt.active {
    background: color-mix(in srgb, var(--primary) 15%, transparent);
  }

  .opt:hover .name,
  .opt:hover .ico,
  .opt:hover .effort-val,
  .opt.active .name,
  .opt.active .ico,
  .opt.active .effort-val {
    color: var(--primary);
  }

  .check {
    display: flex;
    color: var(--primary);
    flex-shrink: 0;
  }

  .effort-check {
    display: flex;
    color: var(--foreground);
    flex-shrink: 0;
  }

  .menu-divider {
    height: 1px;
    background: var(--border);
    margin: 2px 8px;
  }

  .effort-wrapper {
    position: relative;
  }

  .effort-opt {
    padding: 8px 12px;
  }
  
  .effort-val {
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--muted-foreground);
  }

  .effort-menu {
    position: absolute;
    bottom: -8px;
    left: calc(100% + 8px);
    width: 250px;
    background: var(--card);
    border-radius: calc(var(--radius) + 2px);
    box-shadow: var(--shadow-2xl);
    padding: 8px;
    display: flex;
    flex-direction: column;
    cursor: default;
    z-index: 62;
    border: 1px solid var(--border);
  }

  .disclaimer {
    font-size: 11px;
    color: var(--muted-foreground);
    line-height: 1.3;
    margin: 4px 4px 6px;
    text-align: left;
  }

  .effort-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .effort-level-opt {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border: none;
    background: transparent;
    border-radius: var(--radius);
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background 0.12s;
  }

  .effort-level-opt:hover {
    background: color-mix(in srgb, var(--primary) 15%, transparent);
  }

  .effort-level-opt:hover .el-name,
  .effort-level-opt:hover .info-ico {
    color: var(--primary);
  }

  .el-name {
    font-family: var(--font-sans);
    font-size: 12.5px;
    font-weight: 500;
    color: var(--foreground);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .badge {
    background: var(--secondary);
    font-size: 10px;
    padding: 2px 5px;
    border-radius: 4px;
    color: var(--muted-foreground);
  }

  .info-ico {
    color: var(--muted-foreground);
  }

  .thinking-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 4px 4px;
  }

  .tt-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    text-align: left;
  }

  .tt-title {
    font-family: var(--font-sans);
    font-size: 12.5px;
    font-weight: 500;
    color: var(--foreground);
  }

  .tt-sub {
    font-size: 11px;
    color: var(--muted-foreground);
  }

  .switch {
    width: 32px;
    height: 18px;
    background: var(--muted-foreground);
    border-radius: 9px;
    border: none;
    position: relative;
    cursor: pointer;
    transition: background 0.2s;
    opacity: 0.8;
  }

  .switch.on {
    background: var(--primary);
    opacity: 1;
  }

  .switch .knob {
    width: 14px;
    height: 14px;
    background: var(--foreground);
    border-radius: 50%;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: transform 0.2s;
    box-shadow: 0 1px 2px var(--shadow-color);
  }

  .switch.on .knob {
    transform: translateX(14px);
  }

  .ico {
    display: flex;
    flex-shrink: 0;
    color: var(--foreground);
    margin-top: 1px;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    font-family: var(--font-sans);
    font-size: 12.5px;
    font-weight: 600;
    color: var(--foreground);
    line-height: 1.2;
  }

  .sub {
    font-family: var(--font-sans);
    font-size: 10.5px;
    color: var(--muted-foreground);
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
