<script lang="ts">
  import XIcon from "phosphor-svelte/lib/XIcon";
  import ArrowLeftIcon from "phosphor-svelte/lib/ArrowLeftIcon";
  import ArrowUpRightIcon from "phosphor-svelte/lib/ArrowUpRightIcon";
  import KeyIcon from "phosphor-svelte/lib/KeyIcon";
  import BrainIcon from "phosphor-svelte/lib/BrainIcon";
  import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
  import FileTextIcon from "phosphor-svelte/lib/FileTextIcon";
  import ScalesIcon from "phosphor-svelte/lib/ScalesIcon";
  import CheckCircleIcon from "phosphor-svelte/lib/CheckCircleIcon";
  import EyeIcon from "phosphor-svelte/lib/EyeIcon";
  import ProviderIcon from "../../components/ProviderIcon.svelte";
  import SettingsApiKeys from "./SettingsApiKeys.svelte";
  import SettingsMemory from "./SettingsMemory.svelte";
  import { ui, type SettingsSection } from "../../shared/ui.svelte";

  let { onClose }: { onClose: () => void } = $props();

  // ---- Animated hero mockup: a provider key filling in, then clearing,
  // cycling through providers (Anthropic-style "fill and delete" typewriter). ----
  const demoProviders = [
    { id: "anthropic", name: "Anthropic", env: "ANTHROPIC_API_KEY" },
    { id: "openai", name: "OpenAI", env: "OPENAI_API_KEY" },
    { id: "google", name: "Google", env: "GOOGLE_API_KEY" },
  ];
  const KEY_LEN = 28; // dots at full fill

  let provIdx = $state(0);
  let filled = $state(0); // 0..KEY_LEN
  let configured = $state(false);

  let provider = $derived(demoProviders[provIdx]);
  let dots = $derived("•".repeat(filled));

  // Drive the loop only while the launcher is visible (settings grid).
  $effect(() => {
    if (ui.settingsSection !== null) return;

    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;

    const at = (ms: number, fn: () => void) => {
      timer = setTimeout(() => {
        if (!cancelled) fn();
      }, ms);
    };

    const type = () => {
      configured = false;
      if (filled < KEY_LEN) {
        filled += 1;
        at(38 + Math.random() * 34, type);
      } else {
        configured = true;
        at(1400, erase); // hold on the full "configured" key
      }
    };

    const erase = () => {
      configured = false;
      if (filled > 0) {
        filled -= 1;
        at(14, erase);
      } else {
        provIdx = (provIdx + 1) % demoProviders.length;
        at(520, type); // brief pause on the empty field before next provider
      }
    };

    at(500, type);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });

  const titles: Record<Exclude<SettingsSection, null>, string> = {
    "api-keys": "Set up Token",
    memory: "Memory",
    about: "About",
  };

  function open(section: Exclude<SettingsSection, null>) {
    ui.settingsSection = section;
  }
</script>

<div class="settings-page">
  <div class="wrap">
    <header>
      {#if ui.settingsSection}
        <button class="ghost" aria-label="Back" onclick={() => (ui.settingsSection = null)}>
          <ArrowLeftIcon size={16} />
        </button>
        <h1>{titles[ui.settingsSection]}</h1>
      {:else}
        <h1>Settings</h1>
      {/if}
      <button class="ghost close" aria-label="Close" onclick={onClose}>
        <XIcon size={16} />
      </button>
    </header>

    {#if ui.settingsSection === null}
      <div class="bento">
        <!-- Hero: Set up Token -->
        <button class="hero" onclick={() => open("api-keys")}>
          <div class="hero-copy">
            <div class="hero-head">
              <span class="chip-ico"><KeyIcon size={18} weight="fill" /></span>
              <h2>Set up Token</h2>
            </div>
            <p class="lead">Connect a provider so ADE can generate:</p>
            <ul>
              <li><strong>Anthropic</strong> Claude models</li>
              <li><strong>OpenAI</strong>, Google &amp; more</li>
              <li><strong>Keys</strong> stored in your OS keychain</li>
              <li><strong>Switch</strong> models anytime</li>
            </ul>
            <span class="cta">Configure keys</span>
          </div>

          <!-- Animated key mockup: fills then clears, cycling providers -->
          <div class="mock" aria-hidden="true">
            <div class="mock-bar">
              <span class="dot"></span><span class="dot"></span><span class="dot"></span>
            </div>
            <div class="mock-body">
              <div class="mock-label">API Keys</div>

              <div class="mock-row">
                <span class="mock-prov">
                  <ProviderIcon provider={provider.id} size={15} />
                  <span class="mock-name">{provider.name}</span>
                  {#if configured}
                    <span class="mock-check"><CheckCircleIcon size={13} weight="fill" /></span>
                  {/if}
                </span>
                <div class="mock-input">
                  {#if filled === 0}
                    <span class="mock-env">{provider.env}</span>
                  {:else}
                    <span class="mock-dots">{dots}</span>
                  {/if}
                  <span class="mock-caret" class:blink={filled === 0 || configured}></span>
                  <span class="mock-eye"><EyeIcon size={13} /></span>
                </div>
              </div>

              <div class="mock-status" class:on={configured}>
                {#if configured}Saved to keychain{:else}&nbsp;{/if}
              </div>
            </div>
          </div>
        </button>

        <!-- Memory -->
        <section class="panel">
          <div class="panel-head">
            <span class="chip-ico small"><BrainIcon size={16} weight="fill" /></span>
            <div>
              <h2>Memory</h2>
              <p>What ADE has learned from your feedback.</p>
            </div>
          </div>
          <div class="rows">
            <button class="lrow" onclick={() => open("memory")}>
              <FileTextIcon size={16} />
              <span>failures.md</span>
              <ArrowUpRightIcon class="arr" size={14} />
            </button>
            <button class="lrow" onclick={() => open("memory")}>
              <FileTextIcon size={16} />
              <span>preferences.md</span>
              <ArrowUpRightIcon class="arr" size={14} />
            </button>
          </div>
        </section>

        <!-- About -->
        <section class="panel">
          <div class="panel-head">
            <span class="chip-ico small"><SparkleIcon size={16} weight="fill" /></span>
            <div>
              <h2>About</h2>
              <p>GwenLand ADE — Agentic Development Environment.</p>
            </div>
          </div>
          <div class="rows">
            <div class="lrow static">
              <SparkleIcon size={16} />
              <span>Version</span>
              <span class="tag">v0.1.3</span>
            </div>
            <div class="lrow static">
              <ScalesIcon size={16} />
              <span>License</span>
              <span class="tag">MIT</span>
            </div>
          </div>
        </section>
      </div>
    {:else if ui.settingsSection === "api-keys"}
      <SettingsApiKeys />
    {:else if ui.settingsSection === "memory"}
      <SettingsMemory />
    {:else if ui.settingsSection === "about"}
      <div class="about">
        <p class="brand">GwenLand ADE</p>
        <p class="muted">Agentic Development Environment</p>
        <p class="muted ver">v0.1.3 · MIT</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-page {
    position: absolute;
    inset: 0;
    z-index: 10;
    background: var(--background);
    display: flex;
    justify-content: center;
    padding: 28px 24px;
    overflow-y: auto;
  }

  .wrap {
    width: 100%;
    max-width: 920px;
    display: flex;
    flex-direction: column;
  }

  header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 2px 18px;
  }

  header h1 {
    font-family: var(--font-serif);
    font-size: 22px;
    font-weight: 400;
    color: var(--foreground);
    flex: 1;
  }

  .ghost {
    display: flex;
    width: 32px;
    height: 32px;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 6px);
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    flex-shrink: 0;
  }

  .ghost:hover {
    background: color-mix(in srgb, var(--primary) 15%, transparent);
    color: var(--primary);
  }

  /* ---- Bento grid ---- */
  .bento {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .chip-ico {
    display: flex;
    width: 34px;
    height: 34px;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border-radius: calc(var(--radius) - 4px);
    background: color-mix(in srgb, var(--primary) 16%, transparent);
    color: var(--primary);
  }

  .chip-ico.small {
    width: 30px;
    height: 30px;
  }

  h2 {
    font-family: var(--font-sans);
    font-size: 16px;
    font-weight: 600;
    color: var(--foreground);
  }

  /* Hero card spans both columns */
  .hero {
    grid-column: 1 / -1;
    display: flex;
    gap: 24px;
    text-align: left;
    padding: 28px 0 28px 28px; /* no right pad: mock bleeds to the edge */
    background: var(--card);
    border: none;
    border-radius: calc(var(--radius) + 6px);
    box-shadow: var(--shadow-sm);
    cursor: pointer;
    transition: box-shadow 0.18s, background 0.18s;
    overflow: hidden;
    min-height: 340px;
  }

  .hero:hover {
    background: color-mix(in srgb, var(--primary) 6%, var(--card));
    box-shadow: var(--shadow-md);
  }

  .hero-copy {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .hero-head {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 14px;
  }

  .lead {
    font-family: var(--font-sans);
    font-size: 13.5px;
    color: var(--muted-foreground);
    margin-bottom: 10px;
  }

  .hero ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 7px;
    margin-bottom: 18px;
  }

  .hero li {
    position: relative;
    padding-left: 16px;
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--muted-foreground);
  }

  .hero li::before {
    content: "";
    position: absolute;
    left: 3px;
    top: 7px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--primary);
  }

  .hero li strong {
    color: var(--foreground);
    font-weight: 500;
  }

  .cta {
    align-self: flex-start;
    margin-top: auto;
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    color: var(--primary-foreground);
    background: var(--primary);
    padding: 8px 16px;
    border-radius: calc(var(--radius) - 4px);
    transition: background 0.15s;
  }

  .hero:hover .cta {
    background: color-mix(in srgb, var(--primary) 85%, white);
  }

  /* Decorative composer mockup */
  .mock {
    width: 300px;
    flex-shrink: 0;
    align-self: stretch;
    display: flex;
    flex-direction: column;
    background: var(--background);
    /* Bleeds to the card's right/top/bottom edges — the hero's own
       overflow:hidden + radius does the corner clipping, so no radius here. */
    /* Fade the whole panel into the hero card from the left. */
    -webkit-mask-image: linear-gradient(to right, transparent 0%, #000 40%);
    mask-image: linear-gradient(to right, transparent 0%, #000 40%);
    overflow: hidden;
  }

  .mock-bar {
    display: flex;
    gap: 6px;
    padding: 10px 12px;
  }

  .mock-bar .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--muted-foreground);
    opacity: 0.35;
  }

  .mock-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 12px;
    /* Extra left padding keeps the key row clear of the left fade. */
    padding: 10px 22px 22px 44px;
  }

  .mock-label {
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    opacity: 0.7;
  }

  .mock-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mock-prov {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .mock-prov :global(svg) {
    flex-shrink: 0;
  }

  .mock-name {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--foreground);
  }

  .mock-check {
    display: flex;
    color: var(--primary);
  }

  .mock-input {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0;
    background: var(--secondary);
    border-radius: calc(var(--radius) - 6px);
    padding: 8px 30px 8px 10px;
    min-height: 15px;
    overflow: hidden;
  }

  .mock-env {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--muted-foreground);
    opacity: 0.55;
  }

  .mock-dots {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0;
    color: var(--foreground);
    line-height: 1;
    white-space: nowrap;
  }

  .mock-caret {
    width: 1.5px;
    height: 11px;
    background: var(--primary);
    flex-shrink: 0;
    margin-left: 2px;
  }

  .mock-caret.blink {
    animation: caret-blink 1s step-end infinite;
  }

  .mock-eye {
    position: absolute;
    right: 8px;
    display: flex;
    color: var(--muted-foreground);
    opacity: 0.6;
  }

  .mock-status {
    font-family: var(--font-sans);
    font-size: 10px;
    color: var(--primary);
    opacity: 0;
    transition: opacity 0.25s;
    min-height: 12px;
  }

  .mock-status.on {
    opacity: 1;
  }

  @keyframes caret-blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }

  /* ---- Small panels ---- */
  .panel {
    display: flex;
    flex-direction: column;
    padding: 22px;
    background: var(--card);
    border-radius: calc(var(--radius) + 6px);
    box-shadow: var(--shadow-sm);
  }

  .panel-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 16px;
  }

  .panel-head p {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--muted-foreground);
    margin-top: 3px;
    line-height: 1.4;
  }

  .rows {
    display: flex;
    flex-direction: column;
  }

  .lrow {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    text-align: left;
    padding: 11px 4px;
    background: transparent;
    border: none;
    color: var(--foreground);
    font-family: var(--font-sans);
    font-size: 13px;
    cursor: pointer;
    box-shadow: inset 0 -1px 0 color-mix(in srgb, var(--muted-foreground) 12%, transparent);
    transition: color 0.15s;
  }

  .rows .lrow:last-child {
    box-shadow: none;
  }

  .lrow :global(svg) {
    color: var(--muted-foreground);
    flex-shrink: 0;
  }

  .lrow span {
    flex: 1;
  }

  .lrow :global(.arr) {
    opacity: 0;
    transition: opacity 0.15s, transform 0.15s;
  }

  .lrow:hover:not(.static) {
    color: var(--primary);
  }

  .lrow:hover:not(.static) :global(svg) {
    color: var(--primary);
  }

  .lrow:hover:not(.static) :global(.arr) {
    opacity: 1;
    transform: translate(1px, -1px);
  }

  .lrow.static {
    cursor: default;
  }

  .tag {
    flex: none !important;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--muted-foreground);
    background: var(--secondary);
    padding: 2px 8px;
    border-radius: 999px;
  }

  /* ---- About detail ---- */
  .about {
    padding: 8px 2px;
  }

  .brand {
    font-family: var(--font-serif);
    font-size: 22px;
    color: var(--foreground);
  }

  .muted {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--muted-foreground);
    margin-top: 4px;
  }

  .ver {
    font-family: var(--font-mono);
    margin-top: 12px;
    opacity: 0.7;
  }

  @media (max-width: 720px) {
    .bento {
      grid-template-columns: 1fr;
    }
    .hero {
      flex-direction: column;
      padding: 28px;
    }
    .mock {
      width: 100%;
      min-height: 200px;
      border-radius: var(--radius);
      -webkit-mask-image: none;
      mask-image: none;
    }
    .mock-body {
      padding: 10px 18px 22px;
    }
  }
</style>
