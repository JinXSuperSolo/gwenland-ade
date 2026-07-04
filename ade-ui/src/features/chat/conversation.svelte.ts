// Shared conversation state (runes module).
//
// Extracted from Composer so the split-pane layout (GWEN-489) can drive the
// left pane (composer input) and the right pane (preview/output) — and a
// detached preview window — from one source of truth.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Message = {
  role: "user" | "ade";
  content: string;
  prompt?: string;
};

export const chat = $state({
  messages: [] as Message[],
  isStreaming: false,
  lastPrompt: "",
  // Selected provider + model, driven by the picker (defaults set on mount).
  providerId: "",
  modelId: "",
  // Reasoning controls from the picker (M5) — capability-gated per model.
  effort: "high" as "low" | "medium" | "high" | "max",
  thinkingOn: false,
  // Set true after the first generate finishes, so the UI can surface the
  // one-time "detach preview" hint (GWEN-490).
  hasGeneratedOnce: false,
});

// A conversation is "active" once anything has been said.
export function isActive(): boolean {
  return chat.messages.length > 0;
}

let listenersReady = false;

/// Wires the streaming event listeners exactly once, regardless of how many
/// components (or windows) call it. Returns a cleanup fn; the app-level owner
/// should hold it for the app's lifetime.
export function initConversationListeners(): () => void {
  if (listenersReady) return () => {};
  listenersReady = true;

  const unlistenToken = listen<string>("ade://token", (e) => {
    const last = chat.messages[chat.messages.length - 1];
    if (last?.role === "ade") {
      chat.messages[chat.messages.length - 1] = {
        ...last,
        content: last.content + e.payload,
      };
    } else {
      // Tag the response with the prompt that produced it, for feedback.
      chat.messages.push({
        role: "ade",
        content: e.payload,
        prompt: chat.lastPrompt,
      });
    }
  });

  const unlistenDone = listen("ade://done", () => {
    chat.isStreaming = false;
    chat.hasGeneratedOnce = true;
  });

  const unlistenError = listen<string>("ade://error", (e) => {
    chat.isStreaming = false;
    chat.messages.push({ role: "ade", content: `⚠ Error: ${e.payload}` });
  });

  return () => {
    unlistenToken.then((fn) => fn());
    unlistenDone.then((fn) => fn());
    unlistenError.then((fn) => fn());
    listenersReady = false;
  };
}

/// Sends a prompt and starts streaming. No-ops on empty input or while a
/// previous generation is still in flight.
export async function send(text: string): Promise<void> {
  const prompt = text.trim();
  if (!prompt || chat.isStreaming) return;

  chat.messages.push({ role: "user", content: prompt });
  chat.lastPrompt = prompt;
  chat.isStreaming = true;

  try {
    await invoke("generate", { request: { prompt } });
  } catch (err) {
    chat.isStreaming = false;
    chat.messages.push({ role: "ade", content: `⚠ Error: ${err}` });
  }
}
