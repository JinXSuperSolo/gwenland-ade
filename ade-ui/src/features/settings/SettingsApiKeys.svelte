<script lang="ts">
  import { onMount } from "svelte";
  import EyeIcon from "phosphor-svelte/lib/EyeIcon";
  import EyeSlashIcon from "phosphor-svelte/lib/EyeSlashIcon";
  import CheckCircleIcon from "phosphor-svelte/lib/CheckCircleIcon";
  import ProviderIcon from "../../components/ProviderIcon.svelte";
  import {
    listProviders,
    getApiKey,
    saveApiKey,
    type Provider,
  } from "../../shared/providers";

  type Row = {
    provider: Provider;
    value: string;
    revealed: boolean;
    saved: boolean;
    dirty: boolean;
  };

  let rows = $state<Row[]>([]);
  let loading = $state(true);

  // The three providers that shipped first stay grouped at the top; the rest are
  // alphabetical (GWEN-469). Everything is driven from the registry — adding a
  // provider in Rust makes it appear here with no edits.
  const PRIMARY = ["anthropic", "openai", "google"];

  onMount(async () => {
    const providers = await listProviders();
    const primary = PRIMARY.map((id) => providers.find((p) => p.id === id)).filter(
      (p): p is Provider => !!p,
    );
    const rest = providers
      .filter((p) => !PRIMARY.includes(p.id))
      .sort((a, b) => a.name.localeCompare(b.name));

    rows = await Promise.all(
      [...primary, ...rest].map(async (provider) => {
        const existing = await getApiKey(provider.id).catch(() => null);
        return {
          provider,
          value: existing ?? "",
          revealed: false,
          saved: !!existing,
          dirty: false,
        };
      }),
    );
    loading = false;
  });

  async function save(row: Row) {
    await saveApiKey(row.provider.id, row.value);
    row.saved = row.value.trim().length > 0;
    row.dirty = false;
  }

  const primaryRows = $derived(rows.filter((r) => PRIMARY.includes(r.provider.id)));
  const otherRows = $derived(rows.filter((r) => !PRIMARY.includes(r.provider.id)));
</script>

<div class="body">
  {#if loading}
    <p class="muted">Loading providers…</p>
  {:else}
    {#snippet keyRow(row: Row)}
      <div class="row">
        <div class="label">
          <span class="pico"><ProviderIcon provider={row.provider.id} size={15} /></span>
          <span class="name">{row.provider.name}</span>
          {#if row.saved}
            <span class="badge" title="Configured">
              <CheckCircleIcon size={13} weight="fill" />
            </span>
          {/if}
        </div>
        <div class="input-wrap">
          <input
            type={row.revealed ? "text" : "password"}
            placeholder={row.provider.apiKeyEnv}
            bind:value={row.value}
            oninput={() => (row.dirty = true)}
            onblur={() => row.dirty && save(row)}
            autocomplete="off"
            spellcheck="false"
          />
          <button
            class="reveal"
            aria-label={row.revealed ? "Hide" : "Show"}
            onclick={() => (row.revealed = !row.revealed)}
          >
            {#if row.revealed}
              <EyeSlashIcon size={15} />
            {:else}
              <EyeIcon size={15} />
            {/if}
          </button>
        </div>
      </div>
    {/snippet}

    <div class="group-label">Default</div>
    {#each primaryRows as row (row.provider.id)}
      {@render keyRow(row)}
    {/each}

    <div class="group-label" style="margin-top: 18px;">More providers</div>
    {#each otherRows as row (row.provider.id)}
      {@render keyRow(row)}
    {/each}

    <p class="hint">Keys are stored in your OS keychain, never in plain text.</p>
  {/if}
</div>

<style>
  .body {
    padding: 4px 0 12px;
  }

  .group-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted-foreground);
    opacity: 0.7;
    padding: 6px 0 8px;
    user-select: none;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 0;
  }

  .label {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 160px;
    flex-shrink: 0;
  }

  .pico {
    display: flex;
    flex-shrink: 0;
    color: var(--foreground);
    opacity: 0.85;
  }

  .name {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--foreground);
  }

  .badge {
    display: flex;
    color: var(--primary);
  }

  .input-wrap {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
  }

  input {
    width: 100%;
    background: var(--secondary);
    border: none;
    outline: none;
    border-radius: calc(var(--radius) - 4px);
    padding: 8px 34px 8px 12px;
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: var(--foreground);
    transition: box-shadow 0.15s;
  }

  input::placeholder {
    color: var(--muted-foreground);
    font-family: var(--font-mono);
    opacity: 0.6;
  }

  input:focus {
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 40%, transparent);
  }

  .reveal {
    position: absolute;
    right: 6px;
    display: flex;
    width: 24px;
    height: 24px;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 8px);
    color: var(--muted-foreground);
    cursor: pointer;
    transition: color 0.15s;
  }

  .reveal:hover {
    color: var(--foreground);
  }

  .hint,
  .muted {
    display: block;
    margin-top: 16px;
    font-family: var(--font-sans);
    font-size: 11.5px;
    color: var(--muted-foreground);
    opacity: 0.8;
  }
</style>
