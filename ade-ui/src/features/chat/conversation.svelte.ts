// Shared conversation state + the agent tool-loop (runes module).
//
// Drives the real provider stream: it sends the conversation to the selected
// provider with the GL_ tool suite registered, streams text back, and when the
// model calls a tool it runs it via `callTool`, feeds the result back, and
// continues — until the model stops with a plain text answer (M5).

import { invoke } from "@tauri-apps/api/core";
import {
  listProviders,
  getApiKey,
  reasoningParams,
  streamAgent,
  type Provider,
  type ToolCall,
} from "../../shared/providers";
import { GL_TOOLS, callTool } from "../../shared/tools";

// A rendered turn. `tool` blocks show tool activity; `ask` blocks are an
// inline GL_Ask_User prompt (Claude.ai-style) awaiting the user's choice.
export type ToolBlock = {
  kind: "tool";
  name: string;
  input: unknown;
  status: "running" | "done" | "error";
  result?: string;
};
export type AskBlock = {
  kind: "ask";
  id: number;
  question: string;
  options: string[] | null;
  answer?: string;
};
export type Block =
  | { kind: "text"; text: string }
  | ToolBlock
  | AskBlock;

export type Message = {
  role: "user" | "ade";
  content: string; // plain-text mirror (feedback + simple render)
  blocks?: Block[]; // rich blocks for ADE turns
  prompt?: string;
};

export const chat = $state({
  messages: [] as Message[],
  isStreaming: false,
  lastPrompt: "",
  providerId: "",
  modelId: "",
  effort: "high" as "low" | "medium" | "high" | "max",
  thinkingOn: false,
  hasGeneratedOnce: false,
});

let providersCache: Provider[] = [];

export function isActive(): boolean {
  return chat.messages.length > 0;
}

/// Kept for API compatibility with existing callers (App/PreviewWindow). The
/// real streaming is now frontend-driven, so there are no backend events to
/// wire — this is a no-op returning a cleanup fn.
export function initConversationListeners(): () => void {
  return () => {};
}

// --- GL_Ask_User bridge ----------------------------------------------------
//
// The backend's GL_Ask_User emits `ade://ask-user`; we surface it as an inline
// prompt block and resolve it when the user clicks an option (`answerAsk`).

let askListenerReady = false;
async function ensureAskListener() {
  if (askListenerReady) return;
  askListenerReady = true;
  const { onAskUser } = await import("../../shared/tools");
  await onAskUser((req) => {
    const msg = currentAdeMessage();
    if (!msg) return;
    msg.blocks = [
      ...(msg.blocks ?? []),
      { kind: "ask", id: req.id, question: req.question, options: req.options },
    ];
  });
}

/// Answers an inline GL_Ask_User prompt: records the choice and unblocks the
/// waiting backend tool via `gl_answer_user`.
export async function answerAsk(block: AskBlock, answer: string) {
  block.answer = answer;
  await invoke("gl_answer_user", { id: block.id, answer }).catch(() => {});
}

// --- Helpers ---------------------------------------------------------------

function currentAdeMessage(): Message | undefined {
  const last = chat.messages[chat.messages.length - 1];
  return last?.role === "ade" ? last : undefined;
}

/// Appends text to the active ADE message's trailing text block (creating one).
function pushText(text: string) {
  const msg = currentAdeMessage();
  if (!msg) return;
  const blocks = msg.blocks ?? (msg.blocks = []);
  const last = blocks[blocks.length - 1];
  if (last?.kind === "text") last.text += text;
  else blocks.push({ kind: "text", text });
  msg.content += text;
}

// --- Send + tool loop ------------------------------------------------------

export async function send(text: string): Promise<void> {
  const prompt = text.trim();
  if (!prompt || chat.isStreaming) return;

  chat.messages.push({ role: "user", content: prompt });
  chat.lastPrompt = prompt;
  chat.isStreaming = true;

  // Fresh ADE turn to stream into.
  chat.messages.push({ role: "ade", content: "", blocks: [], prompt });

  try {
    await runAgent();
  } catch (err) {
    pushText(`\n\n⚠ Error: ${err instanceof Error ? err.message : String(err)}`);
  } finally {
    chat.isStreaming = false;
    chat.hasGeneratedOnce = true;
  }
}

async function runAgent(): Promise<void> {
  await ensureAskListener();

  if (!providersCache.length) providersCache = await listProviders();
  const provider = providersCache.find((p) => p.id === chat.providerId) ?? providersCache[0];
  const model = provider?.models.find((m) => m.id === chat.modelId) ?? provider?.models[0];
  if (!provider || !model) throw new Error("No model selected");

  // A keychain read can reject on some systems; treat that as "no key" with a
  // clear message rather than a raw error.
  const apiKey = await getApiKey(provider.id).catch(() => null);
  if (!apiKey || !apiKey.trim()) {
    throw new Error(
      `No API key set for ${provider.name}. Open Settings → Set up Token and paste your ${provider.apiKeyEnv}.`,
    );
  }

  const workspace = await invoke<string | null>("get_workspace").catch(() => null);
  const system = buildSystemPrompt(workspace);

  // Provider-neutral running transcript the agent loop appends to.
  const history: import("../../shared/providers").AgentMessage[] = [
    { role: "user", content: chat.lastPrompt },
  ];

  const reasoning = reasoningParams(model, chat.effort, chat.thinkingOn);

  // Bounded tool loop: stream → run any tool calls → feed results → repeat.
  for (let turn = 0; turn < 12; turn++) {
    const toolCalls: ToolCall[] = [];

    await streamAgent({
      provider,
      model: model.id,
      apiKey,
      system,
      messages: history,
      tools: GL_TOOLS,
      reasoning,
      onToken: (t) => pushText(t),
      onToolCall: (tc) => toolCalls.push(tc),
    });

    if (toolCalls.length === 0) return; // model finished with a text answer

    // Record the assistant's tool-call turn, then execute each tool.
    history.push({ role: "assistant", content: "", toolCalls });

    for (const tc of toolCalls) {
      const block: ToolBlock = {
        kind: "tool",
        name: tc.name,
        input: tc.input,
        status: "running",
      };
      const msg = currentAdeMessage();
      msg?.blocks?.push(block);

      try {
        const result = await callTool(tc.name, tc.input as Record<string, unknown>);
        const text = typeof result === "string" ? result : JSON.stringify(result);
        block.status = "done";
        block.result = text;
        history.push({ role: "tool", toolCallId: tc.id, name: tc.name, content: text });
      } catch (e) {
        const text = e instanceof Error ? e.message : String(e);
        block.status = "error";
        block.result = text;
        history.push({ role: "tool", toolCallId: tc.id, name: tc.name, content: `Error: ${text}` });
      }
    }
  }

  pushText("\n\n⚠ Stopped: tool loop exceeded the step limit.");
}

function buildSystemPrompt(workspace: string | null): string {
  return [
    "You are ADE (Agentic Development Environment) by GwenLand — an agentic coding assistant.",
    "The user describes what they want to build; you decide what to do and use the GL_ tools to do it.",
    workspace ? `Current workspace: ${workspace}` : "No workspace selected yet.",
    "Prefer acting with tools over asking. Use GL_Ask_User only when you genuinely need a decision.",
    "Be concise and practical. Respond in Markdown.",
  ].join("\n");
}
