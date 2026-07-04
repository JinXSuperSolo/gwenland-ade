import re

providers_data = [
    ("anthropic", "Anthropic", "ANTHROPIC_API_KEY", "https://api.anthropic.com/v1", "Anthropic", [
        ("claude-sonnet-5", "Claude Sonnet 5", 200_000, 3.0, 15.0),
        ("claude-fable-5", "Claude Fable 5", 200_000, 0.8, 4.0),
        ("claude-opus-5", "Claude Opus 5", 200_000, 15.0, 75.0),
        ("claude-4-5-opus-2025", "Claude 4.5 Opus", 200_000, 15.0, 75.0),
        ("claude-sonnet-4-5", "Claude 4.5 Sonnet", 200_000, 3.0, 15.0),
        ("claude-4-5-haiku-2025", "Claude 4.5 Haiku", 200_000, 0.8, 4.0),
        ("claude-4-opus-2025", "Claude 4.0 Opus", 200_000, 15.0, 75.0),
        ("claude-4-sonnet-2025", "Claude 4.0 Sonnet", 200_000, 3.0, 15.0),
        ("claude-3-7-sonnet-20250219", "Claude 3.7 Sonnet", 200_000, 3.0, 15.0),
        ("claude-3-5-opus-202409", "Claude 3.5 Opus", 200_000, 15.0, 75.0),
        ("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet", 200_000, 3.0, 15.0),
        ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", 200_000, 0.8, 4.0),
        ("claude-3-opus-20240229", "Claude 3 Opus", 200_000, 15.0, 75.0),
        ("claude-3-sonnet-20240229", "Claude 3 Sonnet", 200_000, 3.0, 15.0),
        ("claude-3-haiku-20240307", "Claude 3 Haiku", 200_000, 0.25, 1.25),
    ]),
    ("cohere", "Cohere", "COHERE_API_KEY", "https://api.cohere.com/v2", "OpenAiCompat", [
        ("command-r-plus-2025", "Command R+ 2025", 128_000, 2.5, 10.0),
        ("command-r-2025", "Command R 2025", 128_000, 0.15, 0.6),
        ("command-a", "Command A", 128_000, 1.5, 6.0),
        ("command-r-plus-08-2024", "Command R+ (2024)", 128_000, 2.5, 10.0),
        ("command-r-08-2024", "Command R (2024)", 128_000, 0.15, 0.6),
        ("command-r7b-12-2024", "Command R7B", 128_000, 0.0375, 0.15),
    ]),
    ("deepseek", "DeepSeek", "DEEPSEEK_API_KEY", "https://api.deepseek.com/v1", "OpenAiCompat", [
        ("deepseek-v4-pro", "DeepSeek V4 Pro", 1_000_000, 0.55, 2.19),
        ("deepseek-v4-flash", "DeepSeek V4 Flash", 1_000_000, 0.27, 1.1),
        ("deepseek-reasoner", "DeepSeek Reasoner (R1)", 64_000, 0.55, 2.19),
        ("deepseek-chat", "DeepSeek Chat (V3)", 64_000, 0.27, 1.1),
        ("deepseek-coder", "DeepSeek Coder (V2)", 64_000, 0.14, 0.28),
    ]),
    ("google", "Google", "GEMINI_API_KEY", "https://generativelanguage.googleapis.com/v1beta", "Gemini", [
        ("gemini-3.5-flash", "Gemini 3.5 Flash", 2_000_000, 0.1, 0.4),
        ("gemini-3.1-pro", "Gemini 3.1 Pro", 2_000_000, 1.25, 5.0),
        ("gemini-3-deep-think", "Gemini 3 Deep Think", 1_000_000, 2.0, 8.0),
        ("gemini-2.5-pro", "Gemini 2.5 Pro", 2_000_000, 1.25, 5.0),
        ("gemini-2.0-pro-exp", "Gemini 2.0 Pro Exp", 2_000_000, 1.25, 5.0),
        ("gemini-2.0-flash", "Gemini 2.0 Flash", 1_000_000, 0.1, 0.4),
        ("gemini-1.5-pro", "Gemini 1.5 Pro", 2_000_000, 1.25, 5.0),
        ("gemini-1.5-flash", "Gemini 1.5 Flash", 1_000_000, 0.075, 0.3),
        ("gemini-1.5-flash-8b", "Gemini 1.5 Flash 8B", 1_000_000, 0.0375, 0.15),
        ("gemini-1.0-pro", "Gemini 1.0 Pro", 32_000, 0.5, 1.5),
    ]),
    ("groq", "Groq", "GROQ_API_KEY", "https://api.groq.com/openai/v1", "OpenAiCompat", [
        ("llama-4-70b-versatile", "Llama 4 70B", 128_000, 0.79, 0.99),
        ("llama-4-8b-instant", "Llama 4 8B", 128_000, 0.05, 0.08),
        ("llama-3.3-70b-versatile", "Llama 3.3 70B", 128_000, 0.59, 0.79),
        ("llama-3.1-405b-reasoning", "Llama 3.1 405B", 128_000, 2.0, 2.0),
        ("llama-3.1-70b-versatile", "Llama 3.1 70B", 128_000, 0.59, 0.79),
        ("llama-3.1-8b-instant", "Llama 3.1 8B Instant", 128_000, 0.05, 0.08),
        ("mixtral-8x7b-32768", "Mixtral 8x7B", 32_768, 0.24, 0.24),
        ("gemma2-9b-it", "Gemma 2 9B", 8192, 0.2, 0.2),
        ("deepseek-r1-distill-llama-70b", "DeepSeek R1 Distill 70B", 128_000, 0.75, 0.99),
    ]),
    ("kimi", "Kimi / Moonshot", "MOONSHOT_API_KEY", "https://api.moonshot.cn/v1", "OpenAiCompat", [
        ("moonshot-v2-128k", "Moonshot v2 128K", 128_000, 2.0, 5.0),
        ("moonshot-v1-128k", "Moonshot v1 128K", 128_000, 2.0, 5.0),
        ("moonshot-v1-32k", "Moonshot v1 32K", 32_000, 1.0, 2.5),
        ("moonshot-v1-8k", "Moonshot v1 8K", 8_000, 0.5, 1.25),
    ]),
    ("mistral", "Mistral", "MISTRAL_API_KEY", "https://api.mistral.ai/v1", "OpenAiCompat", [
        ("mistral-large-latest", "Mistral Large (3)", 128_000, 0.5, 1.5),
        ("pixtral-large-latest", "Pixtral Large", 128_000, 0.5, 1.5),
        ("mistral-small-latest", "Mistral Small (4)", 128_000, 0.1, 0.3),
        ("ministral-8b-latest", "Ministral 8B", 128_000, 0.1, 0.1),
        ("ministral-3b-latest", "Ministral 3B", 128_000, 0.04, 0.04),
        ("codestral-latest", "Codestral", 256_000, 0.3, 0.9),
        ("mistral-medium-latest", "Mistral Medium", 32_000, 2.7, 8.1),
    ]),
    ("openai", "OpenAI", "OPENAI_API_KEY", "https://api.openai.com/v1", "OpenAiCompat", [
        ("gpt-5.5", "GPT-5.5", 1_050_000, 5.0, 30.0),
        ("gpt-5", "GPT-5", 1_050_000, 5.0, 30.0),
        ("gpt-4.5-preview", "GPT-4.5 Preview", 128_000, 75.0, 150.0),
        ("o3-pro", "o3 Pro", 200_000, 20.0, 80.0),
        ("o3-mini", "o3-mini", 200_000, 1.1, 4.4),
        ("o1", "o1", 200_000, 15.0, 60.0),
        ("o1-preview", "o1-preview", 128_000, 15.0, 60.0),
        ("o1-mini", "o1-mini", 128_000, 3.0, 12.0),
        ("gpt-4o", "GPT-4o", 128_000, 2.5, 10.0),
        ("gpt-4o-mini", "GPT-4o mini", 128_000, 0.15, 0.6),
        ("gpt-4-turbo", "GPT-4 Turbo", 128_000, 10.0, 30.0),
    ]),
    ("perplexity", "Perplexity", "PERPLEXITY_API_KEY", "https://api.perplexity.ai", "OpenAiCompat", [
        ("sonar-v2-pro", "Sonar v2 Pro", 200_000, 3.0, 15.0),
        ("sonar-v2", "Sonar v2", 200_000, 1.0, 1.0),
        ("sonar-pro", "Sonar Pro", 200_000, 3.0, 15.0),
        ("sonar", "Sonar", 200_000, 1.0, 1.0),
        ("sonar-reasoning-pro", "Sonar Reasoning Pro", 128_000, 2.0, 8.0),
        ("sonar-reasoning", "Sonar Reasoning", 128_000, 1.0, 5.0),
    ]),
    ("qwen", "Qwen / Alibaba Cloud", "DASHSCOPE_API_KEY", "https://dashscope.aliyuncs.com/compatible-mode/v1", "OpenAiCompat", [
        ("qwen-3-max", "Qwen 3 Max", 131_072, 1.6, 6.4),
        ("qwen-3-turbo", "Qwen 3 Turbo", 131_072, 0.4, 1.2),
        ("qwen-max-latest", "Qwen Max Latest", 32_768, 1.6, 6.4),
        ("qwen-plus", "Qwen Plus", 131_072, 0.4, 1.2),
        ("qwen-turbo", "Qwen Turbo", 131_072, 0.1, 0.3),
        ("qwen2.5-coder-32b", "Qwen 2.5 Coder 32B", 32_768, 0.8, 0.8),
    ]),
    ("together", "Together AI", "TOGETHER_API_KEY", "https://api.together.xyz/v1", "OpenAiCompat", [
        ("meta-llama/Llama-3.3-70B-Instruct-Turbo", "Llama 3.3 70B Turbo", 131_072, 0.88, 0.88),
        ("meta-llama/Meta-Llama-3.1-405B-Instruct-Turbo", "Llama 3.1 405B Turbo", 131_072, 3.5, 3.5),
        ("meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo", "Llama 3.1 8B Turbo", 131_072, 0.18, 0.18),
        ("deepseek-ai/DeepSeek-R1", "DeepSeek R1", 163_840, 3.0, 7.0),
        ("Qwen/Qwen2.5-Coder-32B-Instruct", "Qwen 2.5 Coder 32B", 32_768, 0.8, 0.8),
    ]),
    ("xai", "xAI / Grok", "XAI_API_KEY", "https://api.x.ai/v1", "OpenAiCompat", [
        ("grok-4.3", "Grok 4.3", 131_072, 1.25, 2.5),
        ("grok-4.20", "Grok 4.20", 131_072, 1.25, 2.5),
        ("grok-build-0.1", "Grok Build 0.1", 131_072, 1.0, 2.0),
        ("grok-3", "Grok 3", 131_072, 2.0, 10.0),
        ("grok-2-latest", "Grok 2", 131_072, 2.0, 10.0),
        ("grok-1.5", "Grok 1.5", 131_072, 1.5, 5.0),
    ]),
]

def generate_rust_code():
    code = "static REGISTRY: &[Provider] = &[\n"
    for p_id, p_name, p_env, p_url, p_kind, models in providers_data:
        code += f"""    Provider {{
        id: "{p_id}",
        name: "{p_name}",
        api_key_env: "{p_env}",
        base_url: "{p_url}",
        kind: ProviderKind::{p_kind},
        models: &[\n"""
        for m_id, m_name, m_cw, m_ip, m_op in models:
            # properly format floats, e.g. 15.0 or 0.8
            ip_str = f"{m_ip:.4f}".rstrip('0').rstrip('.')
            if '.' not in ip_str: ip_str += ".0"
            op_str = f"{m_op:.4f}".rstrip('0').rstrip('.')
            if '.' not in op_str: op_str += ".0"
            
            code += f"""            Model {{
                id: "{m_id}",
                display_name: "{m_name}",
                context_window: {m_cw:_},
                input_price: {ip_str},
                output_price: {op_str},
            }},\n"""
        code += "        ],\n    },\n"
    code += "];"
    return code

with open("ade-engine/src/providers.rs", "r", encoding="utf-8") as f:
    content = f.read()

# Replace the REGISTRY array
pattern = r"static REGISTRY: &\[Provider\] = &\[.*?\];"
new_code = generate_rust_code()
new_content = re.sub(pattern, new_code, content, flags=re.DOTALL)

with open("ade-engine/src/providers.rs", "w", encoding="utf-8") as f:
    f.write(new_content)

print("Updated providers.rs successfully.")
