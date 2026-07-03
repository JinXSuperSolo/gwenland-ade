<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import type { Component } from "svelte";
  import type { IconComponentProps } from "phosphor-svelte";
  import PlusIcon from "phosphor-svelte/lib/PlusIcon";
  import MicrophoneIcon from "phosphor-svelte/lib/MicrophoneIcon";
  import ArrowUpIcon from "phosphor-svelte/lib/ArrowUpIcon";
  import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
  import CodeIcon from "phosphor-svelte/lib/CodeIcon";
  import MagicWandIcon from "phosphor-svelte/lib/MagicWandIcon";
  import ChartBarIcon from "phosphor-svelte/lib/ChartBarIcon";
  import PenNibIcon from "phosphor-svelte/lib/PenNibIcon";
  import Output from "./Output.svelte";
  import ModelPicker from "./ModelPicker.svelte";

  type Message = { role: "user" | "ade"; content: string; prompt?: string };

  let value = $state("");
  let messages = $state<Message[]>([]);
  // Selected provider + model, driven by the picker (defaults set on mount).
  let providerId = $state("");
  let modelId = $state("");
  let lastPrompt = $state("");
  let isActive = $derived(messages.length > 0);
  let isStreaming = $state(false);
  let canSend = $derived(value.trim().length > 0 && !isStreaming);
  let textarea = $state<HTMLTextAreaElement>();

  let username = $state("");
  invoke<string>("get_username").then((name) => {
    const n = name.trim();
    username = n ? n.charAt(0).toUpperCase() + n.slice(1) : "";
  }).catch(() => {});

  const chips: { label: string; icon: Component<IconComponentProps> }[] = [
    { label: "Code", icon: CodeIcon },
    { label: "Create", icon: MagicWandIcon },
    { label: "Analyze", icon: ChartBarIcon },
    { label: "Write", icon: PenNibIcon },
  ];

  onMount(() => {
    const unlistenToken = listen<string>('ade://token', (e) => {
      const last = messages[messages.length - 1];
      if (last?.role === 'ade') {
        messages[messages.length - 1] = {
          ...last,
          content: last.content + e.payload,
        };
      } else {
        // Tag the response with the prompt that produced it, for feedback.
        messages.push({ role: 'ade', content: e.payload, prompt: lastPrompt });
      }
    });

    const unlistenDone = listen('ade://done', () => {
      isStreaming = false;
    });

    const unlistenError = listen<string>('ade://error', (e) => {
      isStreaming = false;
      messages.push({ role: 'ade', content: `⚠ Error: ${e.payload}` });
    });

    return () => {
      unlistenToken.then(fn => fn());
      unlistenDone.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  });

  async function submit() {
    const text = value.trim();
    if (!text || isStreaming) return;

    messages.push({ role: "user", content: text });
    lastPrompt = text;
    value = "";
    isStreaming = true;
    autoResize();

    try {
      await invoke('generate', { request: { prompt: text } });
    } catch (err) {
      isStreaming = false;
      messages.push({ role: 'ade', content: `⚠ Error: ${err}` });
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  }

  function autoResize() {
    if (!textarea) return;
    textarea.style.height = "auto";
    textarea.style.height = Math.min(textarea.scrollHeight, 300) + "px";
  }
</script>

<div class="layout" class:active={isActive}>
  {#if isActive}
    <Output {messages} {isStreaming} />
  {/if}

  <div class="composer-wrap">
    {#if !isActive}
      <h2 class="idle-title">
        <span class="mark"><SparkleIcon size={30} weight="fill" /></span>
        Hello{username ? `, ${username}` : ""}
      </h2>
    {/if}

    <div class="composer">
      <textarea
        bind:this={textarea}
        bind:value
        onkeydown={handleKeydown}
        oninput={autoResize}
        placeholder={isActive ? "Reply to ADE..." : "How can I help you today?"}
        rows="1"
      ></textarea>

      <div class="footer">
        <div class="side">
          <button class="btn-ghost" aria-label="Attach">
            <PlusIcon size={16} />
          </button>
        </div>
        <div class="side">
          <ModelPicker bind:providerId bind:modelId />
          <button class="btn-ghost" aria-label="Voice input">
            <MicrophoneIcon size={16} />
          </button>
          <button
            class="btn-send"
            onclick={submit}
            disabled={!canSend}
            aria-label="Send"
          >
            <ArrowUpIcon size={15} weight="bold" />
          </button>
        </div>
      </div>
    </div>

    {#if !isActive}
      <div class="chips">
        {#each chips as chip}
          {@const Icon = chip.icon}
          <button class="chip" onclick={() => textarea?.focus()}>
            <Icon size={14} />
            {chip.label}
          </button>
        {/each}
      </div>
    {:else}
      <p class="hint">ADE can make mistakes. Please double-check responses.</p>
    {/if}
  </div>
</div>

<style>
  .layout {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 0 24px 20px;
  }

  .layout.active {
    justify-content: flex-end;
  }

  .idle-title {
    display: flex;
    align-items: center;
    gap: 14px;
    font-family: var(--font-serif);
    font-size: 34px;
    font-weight: 400;
    letter-spacing: var(--tracking-tight);
    color: var(--foreground);
    margin-bottom: 32px;
    text-align: center;
  }

  .mark {
    display: flex;
    color: var(--primary);
  }

  .composer-wrap {
    width: 100%;
    max-width: 720px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .composer {
    width: 100%;
    background: var(--card);
    border-radius: calc(var(--radius) + 8px);
    padding: 16px 20px 12px;
    box-shadow: var(--shadow-sm);
    transition: box-shadow 0.2s ease;
  }

  .composer:focus-within {
    box-shadow: var(--shadow-md);
  }

  textarea {
    width: 100%;
    background: transparent;
    border: none;
    outline: none;
    resize: none;
    font-family: var(--font-sans);
    font-size: 16px;
    letter-spacing: var(--tracking-normal);
    color: var(--foreground);
    line-height: 1.6;
    min-height: 26px;
    max-height: 350px;
    overflow-y: auto;
    padding: 2px 4px;
  }

  textarea::placeholder {
    color: var(--muted-foreground);
  }

  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 12px;
  }

  .side {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .btn-ghost {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: calc(var(--radius) - 6px);
    color: var(--muted-foreground);
    cursor: pointer;
    transition:
      background 0.15s,
      color 0.15s;
  }

  .btn-ghost:hover {
    background: var(--secondary);
    color: var(--foreground);
  }

  .btn-send {
    width: 32px;
    height: 32px;
    margin-left: 4px;
    border-radius: calc(var(--radius) - 6px);
    background: var(--primary);
    border: none;
    color: var(--primary-foreground);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      background 0.15s,
      opacity 0.15s;
  }

  .btn-send:hover:not(:disabled) {
    background: color-mix(in srgb, var(--primary) 85%, white);
  }

  .btn-send:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .chips {
    display: flex;
    gap: 8px;
    margin-top: 12px;
    flex-wrap: wrap;
    justify-content: center;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    background: var(--card);
    border: none;
    border-radius: calc(var(--radius) - 4px);
    color: var(--card-foreground);
    font-family: var(--font-sans);
    font-size: 13px;
    padding: 9px 14px;
    cursor: pointer;
    box-shadow: var(--shadow-xs);
    transition: background 0.15s;
  }

  .chip:hover {
    background: var(--secondary);
  }

  .hint {
    font-size: 11.5px;
    color: var(--muted-foreground);
    opacity: 0.8;
    font-family: var(--font-sans);
  }
</style>
