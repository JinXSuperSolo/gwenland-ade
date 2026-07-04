//! Model provider registry (GWEN-464..469).
//!
//! Typed, static registry of the LLM providers ADE can talk to. The registry is
//! the single source of truth: Settings renders one API-key input per provider
//! from `list_providers`, and the frontend picks a base URL + request shape by
//! `kind` when it streams a completion. Adding a provider is a matter of adding
//! a `Provider` entry here — no Settings-UI edits required.

use serde::Serialize;

/// The wire shape a provider speaks. Chosen so the three request builders in the
/// frontend cover all providers: most are OpenAI-compatible.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// Anthropic Messages API (`/v1/messages`, `x-api-key`, SSE `content_block_delta`).
    Anthropic,
    /// Google Gemini (`generateContent` / `streamGenerateContent`).
    Gemini,
    /// OpenAI-compatible `/chat/completions` with `stream: true`.
    OpenAiCompat,
}

/// A model's reasoning/thinking capability — drives the effort/thinking UI in
/// the model picker and the request parameters the frontend sends (M5).
///
/// Not every model has thinking, and those that do expose it differently:
/// Anthropic uses an `effort` param (or legacy `budget_tokens`), OpenAI o-series
/// use `reasoning_effort`, and Gemini uses `thinking_level` or `thinking_budget`.
/// A few reason unconditionally with no user-facing control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reasoning {
    /// No reasoning/thinking at all (GPT-4o, Llama, Mistral, Cohere, Grok, …).
    None,
    /// Anthropic adaptive effort: low / medium / high / max (Sonnet 4.6+, Opus 4.6+, Sonnet 5).
    Effort,
    /// Anthropic legacy extended thinking via `budget_tokens` (Claude 3.7 and older thinking models).
    BudgetTokens,
    /// Reasons unconditionally, no toggle or level (Fable 5, DeepSeek R1, R1 distills, Gemini 2.5 Pro).
    AlwaysOn,
    /// OpenAI o-series `reasoning_effort`: low / medium / high (o1, o1-mini, o1-preview, o3-mini, o3-pro).
    ReasoningEffort,
    /// Gemini `thinking_level`: low / medium / high / max (Gemini 3.x Pro / Deep Think).
    ThinkingLevel,
    /// Gemini `thinking_budget`, toggleable 0–N tokens (Gemini 2.5 Flash).
    ThinkingBudget,
}

impl Reasoning {
    /// Whether the picker should show a discrete level selector (low/med/high[/max]).
    pub fn has_levels(self) -> bool {
        matches!(
            self,
            Reasoning::Effort | Reasoning::ReasoningEffort | Reasoning::ThinkingLevel
        )
    }
    /// Whether the picker should show an on/off thinking toggle.
    pub fn has_toggle(self) -> bool {
        matches!(self, Reasoning::BudgetTokens | Reasoning::ThinkingBudget)
    }
    /// Whether `max` is a valid level (only Anthropic effort + Gemini thinking_level).
    pub fn has_max(self) -> bool {
        matches!(self, Reasoning::Effort | Reasoning::ThinkingLevel)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Model {
    pub id: &'static str,
    pub display_name: &'static str,
    /// Context window in tokens.
    pub context_window: u32,
    /// USD per 1M input tokens.
    pub input_price: f64,
    /// USD per 1M output tokens.
    pub output_price: f64,
    /// Reasoning/thinking capability (M5).
    pub reasoning: Reasoning,
}

impl Model {
    /// The subtitle shown under a model in the selector:
    /// `"{context} context window · $in/$out per 1M"`.
    pub fn display(&self) -> String {
        format!(
            "{} context window · ${}/${} per 1M",
            humanize_tokens(self.context_window),
            trim_price(self.input_price),
            trim_price(self.output_price),
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    /// Environment variable that also supplies the key (fallback to the keychain).
    pub api_key_env: &'static str,
    pub base_url: &'static str,
    pub kind: ProviderKind,
    pub models: &'static [Model],
}

/// Formats a token count as `128K` / `1M` / `33K`, matching the spec's display.
fn humanize_tokens(n: u32) -> String {
    if n >= 1_000_000 && n.is_multiple_of(1_000_000) {
        format!("{}M", n / 1_000_000)
    } else if n >= 1000 {
        format!("{}K", (n as f64 / 1000.0).round() as u32)
    } else {
        n.to_string()
    }
}

/// `2.5 -> "2.50"`, `0.0375 -> "0.0375"`, `3.0 -> "3"` — drops trailing zeros
/// but keeps cents where they matter.
fn trim_price(p: f64) -> String {
    let s = format!("{p:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

// ---------------------------------------------------------------------------
// Registry seed data
// ---------------------------------------------------------------------------

/// Returns the full provider registry. Order here is the order Settings shows
/// them (existing three first, then the rest alphabetically).
pub fn registry() -> &'static [Provider] {
    REGISTRY
}

/// Looks up a provider by id.
pub fn find(id: &str) -> Option<&'static Provider> {
    REGISTRY.iter().find(|p| p.id == id)
}

static REGISTRY: &[Provider] = &[
    Provider {
        id: "anthropic",
        name: "Anthropic",
        api_key_env: "ANTHROPIC_API_KEY",
        base_url: "https://api.anthropic.com/v1",
        kind: ProviderKind::Anthropic,
        models: &[
            Model {
                id: "claude-sonnet-5",
                display_name: "Claude Sonnet 5",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::Effort,
            },
            Model {
                id: "claude-fable-5",
                display_name: "Claude Fable 5",
                context_window: 200_000,
                input_price: 0.8,
                output_price: 4.0,
                reasoning: Reasoning::AlwaysOn,
            },
            Model {
                id: "claude-opus-5",
                display_name: "Claude Opus 5",
                context_window: 200_000,
                input_price: 15.0,
                output_price: 75.0,
                reasoning: Reasoning::Effort,
            },
            Model {
                id: "claude-4-6-opus-20260205",
                display_name: "Claude 4.6 Opus",
                context_window: 1_000_000,
                input_price: 15.0,
                output_price: 75.0,
                reasoning: Reasoning::Effort,
            },
            Model {
                id: "claude-4-6-sonnet-20260217",
                display_name: "Claude 4.6 Sonnet",
                context_window: 1_000_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::Effort,
            },
            Model {
                id: "claude-4-5-opus-2025",
                display_name: "Claude 4.5 Opus",
                context_window: 200_000,
                input_price: 15.0,
                output_price: 75.0,
                reasoning: Reasoning::BudgetTokens,
            },
            Model {
                id: "claude-sonnet-4-5",
                display_name: "Claude 4.5 Sonnet",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::BudgetTokens,
            },
            Model {
                id: "claude-4-5-haiku-2025",
                display_name: "Claude 4.5 Haiku",
                context_window: 200_000,
                input_price: 0.8,
                output_price: 4.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "claude-4-opus-2025",
                display_name: "Claude 4.0 Opus",
                context_window: 200_000,
                input_price: 15.0,
                output_price: 75.0,
                reasoning: Reasoning::BudgetTokens,
            },
            Model {
                id: "claude-4-sonnet-2025",
                display_name: "Claude 4.0 Sonnet",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::BudgetTokens,
            },
            Model {
                id: "claude-3-7-sonnet-20250219",
                display_name: "Claude 3.7 Sonnet",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::BudgetTokens,
            },
            Model {
                id: "claude-3-5-opus-202409",
                display_name: "Claude 3.5 Opus",
                context_window: 200_000,
                input_price: 15.0,
                output_price: 75.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "claude-3-5-sonnet-20241022",
                display_name: "Claude 3.5 Sonnet",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "claude-3-5-haiku-20241022",
                display_name: "Claude 3.5 Haiku",
                context_window: 200_000,
                input_price: 0.8,
                output_price: 4.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "claude-3-opus-20240229",
                display_name: "Claude 3 Opus",
                context_window: 200_000,
                input_price: 15.0,
                output_price: 75.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "claude-3-sonnet-20240229",
                display_name: "Claude 3 Sonnet",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "claude-3-haiku-20240307",
                display_name: "Claude 3 Haiku",
                context_window: 200_000,
                input_price: 0.25,
                output_price: 1.25,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "cohere",
        name: "Cohere",
        api_key_env: "COHERE_API_KEY",
        base_url: "https://api.cohere.com/v2",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "command-r-plus-2025",
                display_name: "Command R+ 2025",
                context_window: 128_000,
                input_price: 2.5,
                output_price: 10.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "command-r-2025",
                display_name: "Command R 2025",
                context_window: 128_000,
                input_price: 0.15,
                output_price: 0.6,
                reasoning: Reasoning::None,
            },
            Model {
                id: "command-a",
                display_name: "Command A",
                context_window: 128_000,
                input_price: 1.5,
                output_price: 6.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "command-r-plus-08-2024",
                display_name: "Command R+ (2024)",
                context_window: 128_000,
                input_price: 2.5,
                output_price: 10.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "command-r-08-2024",
                display_name: "Command R (2024)",
                context_window: 128_000,
                input_price: 0.15,
                output_price: 0.6,
                reasoning: Reasoning::None,
            },
            Model {
                id: "command-r7b-12-2024",
                display_name: "Command R7B",
                context_window: 128_000,
                input_price: 0.0375,
                output_price: 0.15,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "deepseek",
        name: "DeepSeek",
        api_key_env: "DEEPSEEK_API_KEY",
        base_url: "https://api.deepseek.com/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "deepseek-v4-pro",
                display_name: "DeepSeek V4 Pro",
                context_window: 1_000_000,
                input_price: 0.55,
                output_price: 2.19,
                reasoning: Reasoning::None,
            },
            Model {
                id: "deepseek-v4-flash",
                display_name: "DeepSeek V4 Flash",
                context_window: 1_000_000,
                input_price: 0.27,
                output_price: 1.1,
                reasoning: Reasoning::None,
            },
            Model {
                id: "deepseek-reasoner",
                display_name: "DeepSeek Reasoner (R1)",
                context_window: 64_000,
                input_price: 0.55,
                output_price: 2.19,
                reasoning: Reasoning::AlwaysOn,
            },
            Model {
                id: "deepseek-chat",
                display_name: "DeepSeek Chat (V3)",
                context_window: 64_000,
                input_price: 0.27,
                output_price: 1.1,
                reasoning: Reasoning::None,
            },
            Model {
                id: "deepseek-coder",
                display_name: "DeepSeek Coder (V2)",
                context_window: 64_000,
                input_price: 0.14,
                output_price: 0.28,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "google",
        name: "Google",
        api_key_env: "GEMINI_API_KEY",
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        kind: ProviderKind::Gemini,
        models: &[
            Model {
                id: "gemini-3.5-flash",
                display_name: "Gemini 3.5 Flash",
                context_window: 2_000_000,
                input_price: 0.1,
                output_price: 0.4,
                reasoning: Reasoning::ThinkingBudget,
            },
            Model {
                id: "gemini-3.1-pro",
                display_name: "Gemini 3.1 Pro",
                context_window: 2_000_000,
                input_price: 1.25,
                output_price: 5.0,
                reasoning: Reasoning::ThinkingLevel,
            },
            Model {
                id: "gemini-3-deep-think",
                display_name: "Gemini 3 Deep Think",
                context_window: 1_000_000,
                input_price: 2.0,
                output_price: 8.0,
                reasoning: Reasoning::ThinkingLevel,
            },
            Model {
                id: "gemini-2.5-pro",
                display_name: "Gemini 2.5 Pro",
                context_window: 2_000_000,
                input_price: 1.25,
                output_price: 5.0,
                reasoning: Reasoning::AlwaysOn,
            },
            Model {
                id: "gemini-2.5-flash",
                display_name: "Gemini 2.5 Flash",
                context_window: 1_000_000,
                input_price: 0.3,
                output_price: 2.5,
                reasoning: Reasoning::ThinkingBudget,
            },
            Model {
                id: "gemini-2.0-pro-exp",
                display_name: "Gemini 2.0 Pro Exp",
                context_window: 2_000_000,
                input_price: 1.25,
                output_price: 5.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gemini-2.0-flash",
                display_name: "Gemini 2.0 Flash",
                context_window: 1_000_000,
                input_price: 0.1,
                output_price: 0.4,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gemini-1.5-pro",
                display_name: "Gemini 1.5 Pro",
                context_window: 2_000_000,
                input_price: 1.25,
                output_price: 5.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gemini-1.5-flash",
                display_name: "Gemini 1.5 Flash",
                context_window: 1_000_000,
                input_price: 0.075,
                output_price: 0.3,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gemini-1.5-flash-8b",
                display_name: "Gemini 1.5 Flash 8B",
                context_window: 1_000_000,
                input_price: 0.0375,
                output_price: 0.15,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gemini-1.0-pro",
                display_name: "Gemini 1.0 Pro",
                context_window: 32_000,
                input_price: 0.5,
                output_price: 1.5,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "groq",
        name: "Groq",
        api_key_env: "GROQ_API_KEY",
        base_url: "https://api.groq.com/openai/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "llama-4-70b-versatile",
                display_name: "Llama 4 70B",
                context_window: 128_000,
                input_price: 0.79,
                output_price: 0.99,
                reasoning: Reasoning::None,
            },
            Model {
                id: "llama-4-8b-instant",
                display_name: "Llama 4 8B",
                context_window: 128_000,
                input_price: 0.05,
                output_price: 0.08,
                reasoning: Reasoning::None,
            },
            Model {
                id: "llama-3.3-70b-versatile",
                display_name: "Llama 3.3 70B",
                context_window: 128_000,
                input_price: 0.59,
                output_price: 0.79,
                reasoning: Reasoning::None,
            },
            Model {
                id: "llama-3.1-405b-reasoning",
                display_name: "Llama 3.1 405B",
                context_window: 128_000,
                input_price: 2.0,
                output_price: 2.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "llama-3.1-70b-versatile",
                display_name: "Llama 3.1 70B",
                context_window: 128_000,
                input_price: 0.59,
                output_price: 0.79,
                reasoning: Reasoning::None,
            },
            Model {
                id: "llama-3.1-8b-instant",
                display_name: "Llama 3.1 8B Instant",
                context_window: 128_000,
                input_price: 0.05,
                output_price: 0.08,
                reasoning: Reasoning::None,
            },
            Model {
                id: "mixtral-8x7b-32768",
                display_name: "Mixtral 8x7B",
                context_window: 32_768,
                input_price: 0.24,
                output_price: 0.24,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gemma2-9b-it",
                display_name: "Gemma 2 9B",
                context_window: 8_192,
                input_price: 0.2,
                output_price: 0.2,
                reasoning: Reasoning::None,
            },
            Model {
                id: "deepseek-r1-distill-llama-70b",
                display_name: "DeepSeek R1 Distill 70B",
                context_window: 128_000,
                input_price: 0.75,
                output_price: 0.99,
                reasoning: Reasoning::AlwaysOn,
            },
        ],
    },
    Provider {
        // Kimi is the AI; Moonshot AI is the company behind it. The product name
        // is "Kimi" — the API identifiers use it too on the international endpoint.
        id: "kimi",
        name: "Kimi",
        api_key_env: "KIMI_API_KEY",
        base_url: "https://api.moonshot.ai/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            // Frontier K2 line (256K context, 1T-param MoE). Newest first.
            Model {
                id: "kimi-k2-6",
                display_name: "Kimi K2.6",
                context_window: 256_000,
                input_price: 0.6,
                output_price: 2.5,
                reasoning: Reasoning::None,
            },
            Model {
                id: "kimi-k2-6-thinking",
                display_name: "Kimi K2.6 Thinking",
                context_window: 256_000,
                input_price: 0.6,
                output_price: 2.5,
                // K2 Thinking reasons unconditionally — no toggle exposed.
                reasoning: Reasoning::AlwaysOn,
            },
            Model {
                id: "kimi-k2-5",
                display_name: "Kimi K2.5",
                context_window: 256_000,
                input_price: 0.6,
                output_price: 2.5,
                reasoning: Reasoning::None,
            },
            Model {
                id: "kimi-k2-thinking",
                display_name: "Kimi K2 Thinking",
                context_window: 256_000,
                input_price: 0.6,
                output_price: 2.5,
                reasoning: Reasoning::AlwaysOn,
            },
            Model {
                id: "kimi-k2-0905-preview",
                display_name: "Kimi K2 (0905)",
                context_window: 256_000,
                input_price: 0.6,
                output_price: 2.5,
                reasoning: Reasoning::None,
            },
            Model {
                id: "kimi-k2-turbo-preview",
                display_name: "Kimi K2 Turbo",
                context_window: 256_000,
                input_price: 1.15,
                output_price: 8.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "kimi-k2-instruct",
                display_name: "Kimi K2 Instruct",
                context_window: 128_000,
                input_price: 0.55,
                output_price: 2.2,
                reasoning: Reasoning::None,
            },
            Model {
                id: "kimi-latest",
                display_name: "Kimi Latest",
                context_window: 128_000,
                input_price: 2.0,
                output_price: 5.0,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "mistral",
        name: "Mistral",
        api_key_env: "MISTRAL_API_KEY",
        base_url: "https://api.mistral.ai/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "mistral-large-latest",
                display_name: "Mistral Large (3)",
                context_window: 128_000,
                input_price: 0.5,
                output_price: 1.5,
                reasoning: Reasoning::None,
            },
            Model {
                id: "pixtral-large-latest",
                display_name: "Pixtral Large",
                context_window: 128_000,
                input_price: 0.5,
                output_price: 1.5,
                reasoning: Reasoning::None,
            },
            Model {
                id: "mistral-small-latest",
                display_name: "Mistral Small (4)",
                context_window: 128_000,
                input_price: 0.1,
                output_price: 0.3,
                reasoning: Reasoning::None,
            },
            Model {
                id: "ministral-8b-latest",
                display_name: "Ministral 8B",
                context_window: 128_000,
                input_price: 0.1,
                output_price: 0.1,
                reasoning: Reasoning::None,
            },
            Model {
                id: "ministral-3b-latest",
                display_name: "Ministral 3B",
                context_window: 128_000,
                input_price: 0.04,
                output_price: 0.04,
                reasoning: Reasoning::None,
            },
            Model {
                id: "codestral-latest",
                display_name: "Codestral",
                context_window: 256_000,
                input_price: 0.3,
                output_price: 0.9,
                reasoning: Reasoning::None,
            },
            Model {
                id: "mistral-medium-latest",
                display_name: "Mistral Medium",
                context_window: 32_000,
                input_price: 2.7,
                output_price: 8.1,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "openai",
        name: "OpenAI",
        api_key_env: "OPENAI_API_KEY",
        base_url: "https://api.openai.com/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "gpt-5.5",
                display_name: "GPT-5.5",
                context_window: 1_050_000,
                input_price: 5.0,
                output_price: 30.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gpt-5",
                display_name: "GPT-5",
                context_window: 1_050_000,
                input_price: 5.0,
                output_price: 30.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gpt-4.5-preview",
                display_name: "GPT-4.5 Preview",
                context_window: 128_000,
                input_price: 75.0,
                output_price: 150.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "o3-pro",
                display_name: "o3 Pro",
                context_window: 200_000,
                input_price: 20.0,
                output_price: 80.0,
                reasoning: Reasoning::ReasoningEffort,
            },
            Model {
                id: "o3-mini",
                display_name: "o3-mini",
                context_window: 200_000,
                input_price: 1.1,
                output_price: 4.4,
                reasoning: Reasoning::ReasoningEffort,
            },
            Model {
                id: "o1",
                display_name: "o1",
                context_window: 200_000,
                input_price: 15.0,
                output_price: 60.0,
                reasoning: Reasoning::ReasoningEffort,
            },
            Model {
                id: "o1-preview",
                display_name: "o1-preview",
                context_window: 128_000,
                input_price: 15.0,
                output_price: 60.0,
                reasoning: Reasoning::ReasoningEffort,
            },
            Model {
                id: "o1-mini",
                display_name: "o1-mini",
                context_window: 128_000,
                input_price: 3.0,
                output_price: 12.0,
                reasoning: Reasoning::ReasoningEffort,
            },
            Model {
                id: "gpt-4o",
                display_name: "GPT-4o",
                context_window: 128_000,
                input_price: 2.5,
                output_price: 10.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gpt-4o-mini",
                display_name: "GPT-4o mini",
                context_window: 128_000,
                input_price: 0.15,
                output_price: 0.6,
                reasoning: Reasoning::None,
            },
            Model {
                id: "gpt-4-turbo",
                display_name: "GPT-4 Turbo",
                context_window: 128_000,
                input_price: 10.0,
                output_price: 30.0,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "perplexity",
        name: "Perplexity",
        api_key_env: "PERPLEXITY_API_KEY",
        base_url: "https://api.perplexity.ai",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "sonar-v2-pro",
                display_name: "Sonar v2 Pro",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "sonar-v2",
                display_name: "Sonar v2",
                context_window: 200_000,
                input_price: 1.0,
                output_price: 1.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "sonar-pro",
                display_name: "Sonar Pro",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "sonar",
                display_name: "Sonar",
                context_window: 200_000,
                input_price: 1.0,
                output_price: 1.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "sonar-reasoning-pro",
                display_name: "Sonar Reasoning Pro",
                context_window: 128_000,
                input_price: 2.0,
                output_price: 8.0,
                reasoning: Reasoning::AlwaysOn,
            },
            Model {
                id: "sonar-reasoning",
                display_name: "Sonar Reasoning",
                context_window: 128_000,
                input_price: 1.0,
                output_price: 5.0,
                reasoning: Reasoning::AlwaysOn,
            },
        ],
    },
    Provider {
        id: "qwen",
        name: "Qwen / Alibaba Cloud",
        api_key_env: "DASHSCOPE_API_KEY",
        base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "qwen-3-max",
                display_name: "Qwen 3 Max",
                context_window: 131_072,
                input_price: 1.6,
                output_price: 6.4,
                reasoning: Reasoning::None,
            },
            Model {
                id: "qwen-3-turbo",
                display_name: "Qwen 3 Turbo",
                context_window: 131_072,
                input_price: 0.4,
                output_price: 1.2,
                reasoning: Reasoning::None,
            },
            Model {
                id: "qwen-max-latest",
                display_name: "Qwen Max Latest",
                context_window: 32_768,
                input_price: 1.6,
                output_price: 6.4,
                reasoning: Reasoning::None,
            },
            Model {
                id: "qwen-plus",
                display_name: "Qwen Plus",
                context_window: 131_072,
                input_price: 0.4,
                output_price: 1.2,
                reasoning: Reasoning::None,
            },
            Model {
                id: "qwen-turbo",
                display_name: "Qwen Turbo",
                context_window: 131_072,
                input_price: 0.1,
                output_price: 0.3,
                reasoning: Reasoning::None,
            },
            Model {
                id: "qwen2.5-coder-32b",
                display_name: "Qwen 2.5 Coder 32B",
                context_window: 32_768,
                input_price: 0.8,
                output_price: 0.8,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "together",
        name: "Together AI",
        api_key_env: "TOGETHER_API_KEY",
        base_url: "https://api.together.xyz/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
                display_name: "Llama 3.3 70B Turbo",
                context_window: 131_072,
                input_price: 0.88,
                output_price: 0.88,
                reasoning: Reasoning::None,
            },
            Model {
                id: "meta-llama/Meta-Llama-3.1-405B-Instruct-Turbo",
                display_name: "Llama 3.1 405B Turbo",
                context_window: 131_072,
                input_price: 3.5,
                output_price: 3.5,
                reasoning: Reasoning::None,
            },
            Model {
                id: "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo",
                display_name: "Llama 3.1 8B Turbo",
                context_window: 131_072,
                input_price: 0.18,
                output_price: 0.18,
                reasoning: Reasoning::None,
            },
            Model {
                id: "deepseek-ai/DeepSeek-R1",
                display_name: "DeepSeek R1",
                context_window: 163_840,
                input_price: 3.0,
                output_price: 7.0,
                reasoning: Reasoning::AlwaysOn,
            },
            Model {
                id: "Qwen/Qwen2.5-Coder-32B-Instruct",
                display_name: "Qwen 2.5 Coder 32B",
                context_window: 32_768,
                input_price: 0.8,
                output_price: 0.8,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        id: "xai",
        name: "xAI / Grok",
        api_key_env: "XAI_API_KEY",
        base_url: "https://api.x.ai/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "grok-4.3",
                display_name: "Grok 4.3",
                context_window: 131_072,
                input_price: 1.25,
                output_price: 2.5,
                reasoning: Reasoning::None,
            },
            Model {
                id: "grok-4.20",
                display_name: "Grok 4.20",
                context_window: 131_072,
                input_price: 1.25,
                output_price: 2.5,
                reasoning: Reasoning::None,
            },
            Model {
                id: "grok-build-0.1",
                display_name: "Grok Build 0.1",
                context_window: 131_072,
                input_price: 1.0,
                output_price: 2.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "grok-3",
                display_name: "Grok 3",
                context_window: 131_072,
                input_price: 2.0,
                output_price: 10.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "grok-2-latest",
                display_name: "Grok 2",
                context_window: 131_072,
                input_price: 2.0,
                output_price: 10.0,
                reasoning: Reasoning::None,
            },
            Model {
                id: "grok-1.5",
                display_name: "Grok 1.5",
                context_window: 131_072,
                input_price: 1.5,
                output_price: 5.0,
                reasoning: Reasoning::None,
            },
        ],
    },
    Provider {
        // Z.ai (formerly "Ziphu", GWEN-464) — Zhipu AI's GLM models, now branded
        // Z.ai, on an OpenAI-compatible endpoint.
        id: "zai",
        name: "Z.ai",
        api_key_env: "ZAI_API_KEY",
        base_url: "https://api.z.ai/api/openai/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "glm-4.6",
                display_name: "GLM-4.6",
                context_window: 200_000,
                input_price: 0.6,
                output_price: 2.2,
                // GLM-4.6 exposes a toggleable thinking mode (OpenAI-compat
                // `thinking` param), closest to a thinking on/off budget.
                reasoning: Reasoning::ThinkingBudget,
            },
            Model {
                id: "glm-4.5",
                display_name: "GLM-4.5",
                context_window: 128_000,
                input_price: 0.6,
                output_price: 2.2,
                reasoning: Reasoning::ThinkingBudget,
            },
            Model {
                id: "glm-4.5-air",
                display_name: "GLM-4.5 Air",
                context_window: 128_000,
                input_price: 0.2,
                output_price: 1.1,
                reasoning: Reasoning::ThinkingBudget,
            },
            Model {
                id: "glm-4.5-flash",
                display_name: "GLM-4.5 Flash",
                context_window: 128_000,
                input_price: 0.0,
                output_price: 0.0,
                reasoning: Reasoning::None,
            },
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_matches_spec() {
        assert_eq!(humanize_tokens(128_000), "128K");
        assert_eq!(humanize_tokens(1_000_000), "1M");
        assert_eq!(humanize_tokens(32_768), "33K");
        assert_eq!(humanize_tokens(8192), "8K");
    }

    #[test]
    fn price_trims_zeros_but_keeps_cents() {
        assert_eq!(trim_price(2.5), "2.5");
        assert_eq!(trim_price(3.0), "3");
        assert_eq!(trim_price(0.0375), "0.0375");
        assert_eq!(trim_price(0.15), "0.15");
    }

    #[test]
    fn model_display_format() {
        let m = Model {
            id: "x",
            display_name: "X",
            context_window: 128_000,
            input_price: 2.5,
            output_price: 10.0,
            reasoning: Reasoning::None,
        };
        assert_eq!(m.display(), "128K context window · $2.5/$10 per 1M");
    }

    #[test]
    fn registry_ids_are_unique() {
        let mut ids: Vec<_> = registry().iter().map(|p| p.id).collect();
        ids.sort();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate provider id in registry");
    }

    #[test]
    fn spec_providers_all_present() {
        // Z.ai (formerly "Ziphu", GWEN-464) is the merged `zai` entry.
        for id in [
            "anthropic",
            "openai",
            "google",
            "deepseek",
            "qwen",
            "zai",
            "kimi",
            "mistral",
            "xai",
            "groq",
            "cohere",
            "perplexity",
            "together",
        ] {
            assert!(find(id).is_some(), "missing provider: {id}");
        }
    }

    /// Spot-check the reasoning capability map (M5): the right models get the
    /// right control, and plain chat models get none.
    #[test]
    fn reasoning_capabilities_are_mapped() {
        let model = |pid: &str, mid: &str| {
            find(pid)
                .and_then(|p| p.models.iter().find(|m| m.id == mid))
                .unwrap_or_else(|| panic!("missing {pid}/{mid}"))
                .reasoning
        };

        // Anthropic: adaptive effort vs legacy budget vs always-on vs none.
        assert_eq!(model("anthropic", "claude-sonnet-5"), Reasoning::Effort);
        assert_eq!(
            model("anthropic", "claude-3-7-sonnet-20250219"),
            Reasoning::BudgetTokens
        );
        assert_eq!(model("anthropic", "claude-fable-5"), Reasoning::AlwaysOn);
        assert_eq!(
            model("anthropic", "claude-3-5-haiku-20241022"),
            Reasoning::None
        );

        // OpenAI: o-series get reasoning_effort, GPT-4o does not.
        assert_eq!(model("openai", "o1"), Reasoning::ReasoningEffort);
        assert_eq!(model("openai", "o3-mini"), Reasoning::ReasoningEffort);
        assert_eq!(model("openai", "gpt-4o"), Reasoning::None);

        // Gemini: thinking_level vs toggleable budget vs always-on.
        assert_eq!(model("google", "gemini-3.1-pro"), Reasoning::ThinkingLevel);
        assert_eq!(
            model("google", "gemini-2.5-flash"),
            Reasoning::ThinkingBudget
        );
        assert_eq!(model("google", "gemini-2.5-pro"), Reasoning::AlwaysOn);

        // Reasoner models on OpenAI-compat providers are always-on.
        assert_eq!(model("deepseek", "deepseek-reasoner"), Reasoning::AlwaysOn);

        // Derived UI flags.
        assert!(Reasoning::Effort.has_levels() && Reasoning::Effort.has_max());
        assert!(Reasoning::ReasoningEffort.has_levels() && !Reasoning::ReasoningEffort.has_max());
        assert!(Reasoning::ThinkingBudget.has_toggle());
        assert!(!Reasoning::AlwaysOn.has_levels() && !Reasoning::AlwaysOn.has_toggle());
    }
}
