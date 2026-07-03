<script lang="ts">
  import { onMount } from "svelte";
  import ProviderIcon from "./ProviderIcon.svelte";
  import { listProviders, type Provider, type Model } from "./providers";

  // Two-way bound selection: `{ providerId, modelId }`.
  let {
    providerId = $bindable(),
    modelId = $bindable(),
  }: { providerId: string; modelId: string } = $props();

  let providers = $state<Provider[]>([]);
  let open = $state(false);

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
            <span class="ico"><ProviderIcon provider={p.id} size={19} /></span>
            <span class="text">
              <span class="name">{m.displayName}</span>
              <span class="sub">{m.display}</span>
            </span>
          </button>
        {/each}
      {/each}
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
    background: var(--secondary);
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

  /* Opens upward from the composer button, matching the mockup. */
  .menu {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    z-index: 61;
    width: 320px;
    max-height: 380px;
    overflow-y: auto;
    background: var(--card);
    border-radius: calc(var(--radius) + 2px);
    box-shadow: var(--shadow-2xl);
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .opt {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 9px 10px;
    border: none;
    background: transparent;
    border-radius: var(--radius);
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background 0.12s;
  }

  .opt:hover {
    background: var(--secondary);
  }

  .opt.active {
    background: var(--secondary);
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
    font-size: 13.5px;
    font-weight: 600;
    color: var(--foreground);
    line-height: 1.2;
  }

  .sub {
    font-family: var(--font-sans);
    font-size: 11.5px;
    color: var(--muted-foreground);
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
