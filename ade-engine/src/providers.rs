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
                id: "claude-opus-4-8",
                display_name: "Claude Opus 4.8",
                context_window: 1_000_000,
                input_price: 5.0,
                output_price: 25.0,
            },
            Model {
                id: "claude-sonnet-5",
                display_name: "Claude Sonnet 5",
                context_window: 1_000_000,
                input_price: 3.0,
                output_price: 15.0,
            },
            Model {
                id: "claude-haiku-4-5",
                display_name: "Claude Haiku 4.5",
                context_window: 200_000,
                input_price: 1.0,
                output_price: 5.0,
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
                id: "gpt-4o",
                display_name: "GPT-4o",
                context_window: 128_000,
                input_price: 2.5,
                output_price: 10.0,
            },
            Model {
                id: "gpt-4o-mini",
                display_name: "GPT-4o mini",
                context_window: 128_000,
                input_price: 0.15,
                output_price: 0.6,
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
                id: "gemini-2.0-flash",
                display_name: "Gemini 2.0 Flash",
                context_window: 1_000_000,
                input_price: 0.1,
                output_price: 0.4,
            },
            Model {
                id: "gemini-1.5-pro",
                display_name: "Gemini 1.5 Pro",
                context_window: 2_000_000,
                input_price: 1.25,
                output_price: 5.0,
            },
        ],
    },
    // --- Existing providers already in the registry (per spec) ---
    Provider {
        id: "cohere",
        name: "Cohere",
        api_key_env: "COHERE_API_KEY",
        base_url: "https://api.cohere.com/v2",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "command-r-plus-08-2024",
                display_name: "Command R+",
                context_window: 128_000,
                input_price: 2.5,
                output_price: 10.0,
            },
            Model {
                id: "command-r-08-2024",
                display_name: "Command R",
                context_window: 128_000,
                input_price: 0.15,
                output_price: 0.6,
            },
            Model {
                id: "command-r7b-12-2024",
                display_name: "Command R7B",
                context_window: 128_000,
                input_price: 0.0375,
                output_price: 0.15,
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
                id: "deepseek-chat",
                display_name: "DeepSeek Chat",
                context_window: 64_000,
                input_price: 0.27,
                output_price: 1.1,
            },
            Model {
                id: "deepseek-reasoner",
                display_name: "DeepSeek Reasoner",
                context_window: 64_000,
                input_price: 0.55,
                output_price: 2.19,
            },
        ],
    },
    Provider {
        id: "glm",
        name: "GLM / Z.AI",
        api_key_env: "GLM_API_KEY",
        base_url: "https://open.bigmodel.cn/api/paas/v4",
        kind: ProviderKind::OpenAiCompat,
        models: &[Model {
            id: "glm-4-plus",
            display_name: "GLM-4-Plus",
            context_window: 128_000,
            input_price: 0.6,
            output_price: 0.6,
        }],
    },
    Provider {
        id: "groq",
        name: "Groq",
        api_key_env: "GROQ_API_KEY",
        base_url: "https://api.groq.com/openai/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[
            Model {
                id: "llama-3.3-70b-versatile",
                display_name: "Llama 3.3 70B",
                context_window: 128_000,
                input_price: 0.59,
                output_price: 0.79,
            },
            Model {
                id: "llama-3.1-8b-instant",
                display_name: "Llama 3.1 8B Instant",
                context_window: 128_000,
                input_price: 0.05,
                output_price: 0.08,
            },
            Model {
                id: "mixtral-8x7b-32768",
                display_name: "Mixtral 8x7B",
                context_window: 32_768,
                input_price: 0.24,
                output_price: 0.24,
            },
            Model {
                id: "gemma2-9b-it",
                display_name: "Gemma 2 9B",
                context_window: 8192,
                input_price: 0.2,
                output_price: 0.2,
            },
            Model {
                id: "deepseek-r1-distill-llama-70b",
                display_name: "DeepSeek R1 Distill 70B",
                context_window: 128_000,
                input_price: 0.75,
                output_price: 0.99,
            },
        ],
    },
    Provider {
        id: "kimi",
        name: "Kimi / Moonshot",
        api_key_env: "MOONSHOT_API_KEY",
        base_url: "https://api.moonshot.cn/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[Model {
            id: "moonshot-v1-128k",
            display_name: "Moonshot v1 128K",
            context_window: 128_000,
            input_price: 2.0,
            output_price: 5.0,
        }],
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
                display_name: "Mistral Large",
                context_window: 128_000,
                input_price: 2.0,
                output_price: 6.0,
            },
            Model {
                id: "mistral-small-latest",
                display_name: "Mistral Small",
                context_window: 128_000,
                input_price: 0.2,
                output_price: 0.6,
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
                id: "sonar-pro",
                display_name: "Sonar Pro",
                context_window: 200_000,
                input_price: 3.0,
                output_price: 15.0,
            },
            Model {
                id: "sonar",
                display_name: "Sonar",
                context_window: 200_000,
                input_price: 1.0,
                output_price: 1.0,
            },
            Model {
                id: "sonar-reasoning-pro",
                display_name: "Sonar Reasoning Pro",
                context_window: 128_000,
                input_price: 2.0,
                output_price: 8.0,
            },
            Model {
                id: "sonar-reasoning",
                display_name: "Sonar Reasoning",
                context_window: 128_000,
                input_price: 1.0,
                output_price: 5.0,
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
                id: "qwen-max",
                display_name: "Qwen Max",
                context_window: 32_768,
                input_price: 1.6,
                output_price: 6.4,
            },
            Model {
                id: "qwen-plus",
                display_name: "Qwen Plus",
                context_window: 131_072,
                input_price: 0.4,
                output_price: 1.2,
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
            },
            Model {
                id: "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo",
                display_name: "Llama 3.1 8B Turbo",
                context_window: 131_072,
                input_price: 0.18,
                output_price: 0.18,
            },
            Model {
                id: "deepseek-ai/DeepSeek-R1",
                display_name: "DeepSeek R1",
                context_window: 163_840,
                input_price: 3.0,
                output_price: 7.0,
            },
            Model {
                id: "Qwen/Qwen2.5-Coder-32B-Instruct",
                display_name: "Qwen 2.5 Coder 32B",
                context_window: 32_768,
                input_price: 0.8,
                output_price: 0.8,
            },
        ],
    },
    Provider {
        id: "xai",
        name: "xAI / Grok",
        api_key_env: "XAI_API_KEY",
        base_url: "https://api.x.ai/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[Model {
            id: "grok-2-latest",
            display_name: "Grok 2",
            context_window: 131_072,
            input_price: 2.0,
            output_price: 10.0,
        }],
    },
    // GWEN-464: Ziphu. Base URL + model data are placeholders pending verification
    // against Ziphu's official docs/pricing at implementation time.
    Provider {
        id: "ziphu",
        name: "Ziphu",
        api_key_env: "ZIPHU_API_KEY",
        base_url: "https://api.ziphu.com/v1",
        kind: ProviderKind::OpenAiCompat,
        models: &[Model {
            id: "ziphu-chat",
            display_name: "Ziphu Chat",
            context_window: 128_000,
            input_price: 0.5,
            output_price: 1.5,
        }],
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
        for id in [
            "anthropic",
            "openai",
            "google",
            "deepseek",
            "qwen",
            "glm",
            "kimi",
            "mistral",
            "xai",
            "ziphu",
            "groq",
            "cohere",
            "perplexity",
            "together",
        ] {
            assert!(find(id).is_some(), "missing provider: {id}");
        }
    }
}
