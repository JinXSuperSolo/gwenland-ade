<script lang="ts">
  import { onMount } from "svelte";
  import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
  import { onboarding } from "./onboarding.svelte";

  // Cycling example prompts, animated below the welcome line (GWEN-490).
  const examples = [
    "a login page in React…",
    "a REST API for a todo app…",
    "a CLI that renames files by date…",
    "a dark-mode toggle with no flash…",
  ];
  let idx = $state(0);

  onMount(() => {
    const t = setInterval(() => (idx = (idx + 1) % examples.length), 2600);
    return () => clearInterval(t);
  });
</script>

<div class="onboarding">
  <span class="mark"><SparkleIcon size={28} weight="fill" /></span>
  <h2 class="welcome">Hi, I'm ADE.</h2>
  <p class="sub">Describe what you want to build.</p>

  {#if !onboarding.workspaceChosen}
    <div class="example" aria-live="polite">
      {#key idx}
        <span class="typing">e.g. {examples[idx]}</span>
      {/key}
    </div>
    <p class="cue">Click below to start — I'll ask for your project folder first.</p>
  {/if}
</div>

<style>
  .onboarding {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin-bottom: 28px;
    text-align: center;
  }

  .mark {
    display: flex;
    color: var(--primary);
    margin-bottom: 4px;
  }

  .welcome {
    font-family: var(--font-serif);
    font-size: 30px;
    font-weight: 400;
    letter-spacing: var(--tracking-tight);
    color: var(--foreground);
  }

  .sub {
    font-family: var(--font-sans);
    font-size: 15px;
    color: var(--muted-foreground);
  }

  .example {
    margin-top: 10px;
    height: 22px;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--primary);
    opacity: 0.85;
  }

  .typing {
    display: inline-block;
    animation: rise 0.4s ease;
  }

  .cue {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 0.85;
      transform: translateY(0);
    }
  }
</style>
