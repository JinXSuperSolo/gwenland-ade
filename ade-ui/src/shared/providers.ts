// Provider chat clients — from-scratch `fetch` + SSE streaming (GWEN-464..469).
//
// The Rust registry owns provider metadata and API-key storage; this module owns
// the actual network call. A completion is just: POST JSON to the provider, read
// the streamed response, and hand tokens back to the caller. No SDK, no HTTP
// dependency — the webview's built-in `fetch` + `ReadableStream` do it all.

import { invoke } from "@tauri-apps/api/core";

export type ProviderKind = "anthropic" | "gemini" | "openai_compat";

/// Reasoning/thinking capability, mirrored from the Rust `Reasoning` enum (M5).
export type Reasoning =
  | "none"
  | "effort" // Anthropic adaptive: low/med/high/max
  | "budget_tokens" // Anthropic legacy extended thinking (toggle)
  | "always_on" // reasons unconditionally, no control
  | "reasoning_effort" // OpenAI o-series: low/med/high
  | "thinking_level" // Gemini 3.x: low/med/high/max
  | "thinking_budget"; // Gemini 2.5 Flash / GLM (toggle)

export type Model = {
  id: string;
  displayName: string;
  contextWindow: number;
  inputPrice: number;
  outputPrice: number;
  display: string;
  reasoning: Reasoning;
  /// Show a discrete level selector (low/med/high[/max]).
  reasoningLevels: boolean;
  /// Show an on/off thinking toggle.
  reasoningToggle: boolean;
  /// `max` is a valid level.
  reasoningMax: boolean;
};

/// The reasoning parameter a model expects on the wire, given a chosen level
/// (`"low" | "medium" | "high" | "max"`) and toggle state. Returns a partial
/// request body to merge, or `{}` when the model takes no reasoning control.
export function reasoningParams(
  model: Model,
  level: "low" | "medium" | "high" | "max",
  thinkingOn: boolean,
): Record<string, unknown> {
  switch (model.reasoning) {
    case "effort":
      // Anthropic adaptive effort (Opus 4.6+/Sonnet 4.6+/Sonnet 5).
      return { thinking: { type: "enabled" }, effort: level };
    case "budget_tokens":
      // Anthropic legacy extended thinking — a token budget, off when toggled off.
      return thinkingOn
        ? { thinking: { type: "enabled", budget_tokens: 8192 } }
        : {};
    case "reasoning_effort":
      // OpenAI o-series only accepts low/medium/high (no max).
      return { reasoning_effort: level === "max" ? "high" : level };
    case "thinking_level":
      // Gemini 3.x thinking_level (low/medium/high/max).
      return { generationConfig: { thinkingConfig: { thinkingLevel: level } } };
    case "thinking_budget":
      // Gemini 2.5 Flash / GLM — toggleable budget (0 disables).
      return {
        generationConfig: {
          thinkingConfig: { thinkingBudget: thinkingOn ? -1 : 0 },
        },
      };
    case "always_on":
    case "none":
    default:
      return {};
  }
}

export type Provider = {
  id: string;
  name: string;
  apiKeyEnv: string;
  baseUrl: string;
  kind: ProviderKind;
  models: Model[];
};

export function listProviders(): Promise<Provider[]> {
  return invoke<Provider[]>("list_providers");
}

export function saveApiKey(provider: string, key: string): Promise<void> {
  return invoke("save_api_key", { provider, key });
}

export function getApiKey(provider: string): Promise<string | null> {
  return invoke<string | null>("get_api_key", { provider });
}

export function hasApiKey(provider: string): Promise<boolean> {
  return invoke<boolean>("has_api_key", { provider });
}

export type ChatMessage = { role: "user" | "assistant"; content: string };

export type StreamOptions = {
  provider: Provider;
  model: string;
  system: string;
  messages: ChatMessage[];
  apiKey: string;
  onToken: (text: string) => void;
  signal?: AbortSignal;
};

/// Streams a completion from any provider, dispatching on `provider.kind`.
export async function streamChat(opts: StreamOptions): Promise<void> {
  switch (opts.provider.kind) {
    case "anthropic":
      return streamAnthropic(opts);
    case "gemini":
      return streamGemini(opts);
    case "openai_compat":
      return streamOpenAiCompat(opts);
  }
}

// --- SSE line reader shared by the OpenAI + Anthropic shapes ---------------

async function* sseLines(
  body: ReadableStream<Uint8Array>,
  signal?: AbortSignal,
): AsyncGenerator<string> {
  const reader = body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  try {
    while (true) {
      if (signal?.aborted) return;
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      let nl: number;
      while ((nl = buffer.indexOf("\n")) !== -1) {
        const line = buffer.slice(0, nl).trim();
        buffer = buffer.slice(nl + 1);
        if (line) yield line;
      }
    }
  } finally {
    reader.releaseLock();
  }
}

async function ensureOk(res: Response, provider: string): Promise<void> {
  if (res.ok && res.body) return;
  const detail = await res.text().catch(() => "");
  throw new Error(`${provider} request failed (${res.status}): ${detail.slice(0, 300)}`);
}

// --- Anthropic Messages API ------------------------------------------------

async function streamAnthropic(o: StreamOptions): Promise<void> {
  const res = await fetch(`${o.provider.baseUrl}/messages`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-api-key": o.apiKey,
      "anthropic-version": "2023-06-01",
    },
    body: JSON.stringify({
      model: o.model,
      max_tokens: 4096,
      stream: true,
      system: o.system,
      messages: o.messages,
    }),
    signal: o.signal,
  });
  await ensureOk(res, o.provider.name);

  for await (const line of sseLines(res.body!, o.signal)) {
    if (!line.startsWith("data:")) continue;
    const data = line.slice(5).trim();
    try {
      const evt = JSON.parse(data);
      if (evt.type === "content_block_delta" && evt.delta?.type === "text_delta") {
        o.onToken(evt.delta.text);
      }
    } catch {
      // Ignore keep-alive / non-JSON lines.
    }
  }
}

// --- OpenAI-compatible /chat/completions -----------------------------------

async function streamOpenAiCompat(o: StreamOptions): Promise<void> {
  const messages = [
    ...(o.system ? [{ role: "system", content: o.system }] : []),
    ...o.messages,
  ];
  const res = await fetch(`${o.provider.baseUrl}/chat/completions`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      authorization: `Bearer ${o.apiKey}`,
    },
    body: JSON.stringify({ model: o.model, stream: true, messages }),
    signal: o.signal,
  });
  await ensureOk(res, o.provider.name);

  for await (const line of sseLines(res.body!, o.signal)) {
    if (!line.startsWith("data:")) continue;
    const data = line.slice(5).trim();
    if (data === "[DONE]") return;
    try {
      const evt = JSON.parse(data);
      const delta = evt.choices?.[0]?.delta?.content;
      if (typeof delta === "string" && delta) o.onToken(delta);
    } catch {
      // Ignore.
    }
  }
}

// --- Google Gemini streamGenerateContent -----------------------------------

async function streamGemini(o: StreamOptions): Promise<void> {
  // Gemini streams a JSON array (SSE with `alt=sse`); we use SSE for uniformity.
  const url =
    `${o.provider.baseUrl}/models/${o.model}:streamGenerateContent` +
    `?alt=sse&key=${encodeURIComponent(o.apiKey)}`;
  const contents = o.messages.map((m) => ({
    role: m.role === "assistant" ? "model" : "user",
    parts: [{ text: m.content }],
  }));
  const body: Record<string, unknown> = { contents };
  if (o.system) body.systemInstruction = { parts: [{ text: o.system }] };

  const res = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
    signal: o.signal,
  });
  await ensureOk(res, o.provider.name);

  for await (const line of sseLines(res.body!, o.signal)) {
    if (!line.startsWith("data:")) continue;
    const data = line.slice(5).trim();
    try {
      const evt = JSON.parse(data);
      const text = evt.candidates?.[0]?.content?.parts?.[0]?.text;
      if (typeof text === "string" && text) o.onToken(text);
    } catch {
      // Ignore.
    }
  }
}

// ===========================================================================
// Agent tool-loop layer (M5) — streaming with tool-use across provider kinds.
// ===========================================================================

import type { ToolSchema } from "./tools";

export type ToolCall = { id: string; name: string; input: unknown };

/// Provider-neutral transcript entry the agent loop maintains.
export type AgentMessage =
  | { role: "user"; content: string }
  | { role: "assistant"; content: string; toolCalls?: ToolCall[] }
  | { role: "tool"; toolCallId: string; name: string; content: string };

export type AgentStreamOptions = {
  provider: Provider;
  model: string;
  apiKey: string;
  system: string;
  messages: AgentMessage[];
  tools: ToolSchema[];
  reasoning: Record<string, unknown>;
  onToken: (text: string) => void;
  onToolCall: (call: ToolCall) => void;
  signal?: AbortSignal;
};

/// One streamed turn with tools registered. Emits text via `onToken` and any
/// tool calls via `onToolCall`. Dispatches on the provider's wire shape.
export async function streamAgent(o: AgentStreamOptions): Promise<void> {
  switch (o.provider.kind) {
    case "anthropic":
      return agentAnthropic(o);
    case "openai_compat":
      return agentOpenAiCompat(o);
    case "gemini":
      return agentGemini(o);
  }
}

// --- Anthropic tool-use ----------------------------------------------------

function toAnthropicMessages(msgs: AgentMessage[]): unknown[] {
  const out: unknown[] = [];
  for (const m of msgs) {
    if (m.role === "user") {
      out.push({ role: "user", content: m.content });
    } else if (m.role === "assistant") {
      const content: unknown[] = [];
      if (m.content) content.push({ type: "text", text: m.content });
      for (const tc of m.toolCalls ?? []) {
        content.push({ type: "tool_use", id: tc.id, name: tc.name, input: tc.input ?? {} });
      }
      out.push({ role: "assistant", content });
    } else {
      // tool result → a user turn carrying tool_result blocks
      out.push({
        role: "user",
        content: [{ type: "tool_result", tool_use_id: m.toolCallId, content: m.content }],
      });
    }
  }
  return out;
}

async function agentAnthropic(o: AgentStreamOptions): Promise<void> {
  const tools = o.tools.map((t) => ({
    name: t.name,
    description: t.description,
    input_schema: t.parameters,
  }));

  const res = await fetch(`${o.provider.baseUrl}/messages`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-api-key": o.apiKey,
      "anthropic-version": "2023-06-01",
    },
    body: JSON.stringify({
      model: o.model,
      max_tokens: 4096,
      stream: true,
      system: o.system,
      messages: toAnthropicMessages(o.messages),
      tools,
      ...o.reasoning,
    }),
    signal: o.signal,
  });
  await ensureOk(res, o.provider.name);

  // Accumulate tool_use blocks by index; their input arrives as partial JSON.
  const partial = new Map<number, { id: string; name: string; json: string }>();

  for await (const line of sseLines(res.body!, o.signal)) {
    if (!line.startsWith("data:")) continue;
    const data = line.slice(5).trim();
    try {
      const evt = JSON.parse(data);
      if (evt.type === "content_block_start" && evt.content_block?.type === "tool_use") {
        partial.set(evt.index, {
          id: evt.content_block.id,
          name: evt.content_block.name,
          json: "",
        });
      } else if (evt.type === "content_block_delta") {
        if (evt.delta?.type === "text_delta") o.onToken(evt.delta.text);
        else if (evt.delta?.type === "input_json_delta") {
          const p = partial.get(evt.index);
          if (p) p.json += evt.delta.partial_json ?? "";
        }
      } else if (evt.type === "content_block_stop") {
        const p = partial.get(evt.index);
        if (p) {
          o.onToolCall({ id: p.id, name: p.name, input: safeJson(p.json) });
          partial.delete(evt.index);
        }
      }
    } catch {
      // keep-alive / non-JSON
    }
  }
}

// --- OpenAI-compatible tool_calls ------------------------------------------

function toOpenAiMessages(system: string, msgs: AgentMessage[]): unknown[] {
  const out: unknown[] = [];
  if (system) out.push({ role: "system", content: system });
  for (const m of msgs) {
    if (m.role === "user") out.push({ role: "user", content: m.content });
    else if (m.role === "assistant") {
      out.push({
        role: "assistant",
        content: m.content || null,
        tool_calls: (m.toolCalls ?? []).map((tc) => ({
          id: tc.id,
          type: "function",
          function: { name: tc.name, arguments: JSON.stringify(tc.input ?? {}) },
        })),
      });
    } else {
      out.push({ role: "tool", tool_call_id: m.toolCallId, content: m.content });
    }
  }
  return out;
}

async function agentOpenAiCompat(o: AgentStreamOptions): Promise<void> {
  const tools = o.tools.map((t) => ({
    type: "function",
    function: { name: t.name, description: t.description, parameters: t.parameters },
  }));

  const res = await fetch(`${o.provider.baseUrl}/chat/completions`, {
    method: "POST",
    headers: { "content-type": "application/json", authorization: `Bearer ${o.apiKey}` },
    body: JSON.stringify({
      model: o.model,
      stream: true,
      messages: toOpenAiMessages(o.system, o.messages),
      tools,
      ...o.reasoning,
    }),
    signal: o.signal,
  });
  await ensureOk(res, o.provider.name);

  // tool_calls stream as indexed deltas: id/name on first, arguments appended.
  const acc = new Map<number, { id: string; name: string; args: string }>();

  for await (const line of sseLines(res.body!, o.signal)) {
    if (!line.startsWith("data:")) continue;
    const data = line.slice(5).trim();
    if (data === "[DONE]") break;
    try {
      const evt = JSON.parse(data);
      const delta = evt.choices?.[0]?.delta;
      if (typeof delta?.content === "string" && delta.content) o.onToken(delta.content);
      for (const call of delta?.tool_calls ?? []) {
        const i = call.index ?? 0;
        const cur = acc.get(i) ?? { id: "", name: "", args: "" };
        if (call.id) cur.id = call.id;
        if (call.function?.name) cur.name = call.function.name;
        if (call.function?.arguments) cur.args += call.function.arguments;
        acc.set(i, cur);
      }
    } catch {
      // ignore
    }
  }

  for (const [, c] of [...acc.entries()].sort((a, b) => a[0] - b[0])) {
    if (c.name) o.onToolCall({ id: c.id || c.name, name: c.name, input: safeJson(c.args) });
  }
}

// --- Gemini functionCall ---------------------------------------------------

function toGeminiContents(msgs: AgentMessage[]): unknown[] {
  const out: unknown[] = [];
  for (const m of msgs) {
    if (m.role === "user") out.push({ role: "user", parts: [{ text: m.content }] });
    else if (m.role === "assistant") {
      const parts: unknown[] = [];
      if (m.content) parts.push({ text: m.content });
      for (const tc of m.toolCalls ?? []) {
        parts.push({ functionCall: { name: tc.name, args: tc.input ?? {} } });
      }
      out.push({ role: "model", parts });
    } else {
      out.push({
        role: "user",
        parts: [{ functionResponse: { name: m.name, response: { result: m.content } } }],
      });
    }
  }
  return out;
}

async function agentGemini(o: AgentStreamOptions): Promise<void> {
  const url =
    `${o.provider.baseUrl}/models/${o.model}:streamGenerateContent` +
    `?alt=sse&key=${encodeURIComponent(o.apiKey)}`;

  const functionDeclarations = o.tools.map((t) => ({
    name: t.name,
    description: t.description,
    parameters: t.parameters,
  }));

  const body: Record<string, unknown> = {
    contents: toGeminiContents(o.messages),
    tools: [{ functionDeclarations }],
    systemInstruction: o.system ? { parts: [{ text: o.system }] } : undefined,
    ...o.reasoning,
  };

  const res = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
    signal: o.signal,
  });
  await ensureOk(res, o.provider.name);

  let callSeq = 0;
  for await (const line of sseLines(res.body!, o.signal)) {
    if (!line.startsWith("data:")) continue;
    const data = line.slice(5).trim();
    try {
      const evt = JSON.parse(data);
      for (const part of evt.candidates?.[0]?.content?.parts ?? []) {
        if (typeof part.text === "string" && part.text) o.onToken(part.text);
        if (part.functionCall) {
          o.onToolCall({
            id: `${part.functionCall.name}-${callSeq++}`,
            name: part.functionCall.name,
            input: part.functionCall.args ?? {},
          });
        }
      }
    } catch {
      // ignore
    }
  }
}

function safeJson(s: string): unknown {
  if (!s.trim()) return {};
  try {
    return JSON.parse(s);
  } catch {
    return {};
  }
}
