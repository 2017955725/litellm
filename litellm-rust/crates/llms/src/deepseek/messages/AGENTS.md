# Scope

- DeepSeek's Messages adapter for its Anthropic-compatible endpoint, reached by every model under the `deepseek` provider

# Invariants

- No model-name gating: Python selects this config for any DeepSeek model, so do not add a `claude` substring check
- A caller's `x-api-key` or `authorization` header wins, and the resolved key is only added as `x-api-key` when neither is present
- Only tools with `"type": "custom"` lose their `type` key, after the shared Anthropic transformation. Every other tool passes through untouched
- Thinking blocks in assistant history are forwarded as is

# Boundaries

- Keep DeepSeek differences here instead of adding provider flags to `anthropic/messages` or `base_llm/messages`
- Responses and streams are native Anthropic shape, so reuse the Anthropic response and SSE handling without DeepSeek-specific decoding

# Gotchas

- URL normalization accepts an OpenAI-style base such as `https://api.deepseek.com/v1` and rewrites it to the `/anthropic` surface, so a shared `DEEPSEEK_API_BASE` works for both routes
- DeepSeek does not use Anthropic thinking semantics, so the Claude-specific thinking and conflicting-temperature passes of the shared request policy must be skipped
- Billing metadata is stripped from `system`, and an emptied `system` is omitted
- DeepSeek ignores `anthropic-beta` on Messages, so do not invent DeepSeek-specific betas

# Decisions

- A missing key is a `MissingApiKey` error naming `DEEPSEEK_API_KEY`. Python sends the request without a credential and lets DeepSeek reject it
- Every `anthropic-beta` value is dropped, matching Python's provider filter, which has no `deepseek` column in `anthropic_beta_headers_config.json`

# Known gaps

- Python's last key fallback, the SDK global `litellm.api_key`, is not ported because it is an SDK-only setting
- Not wired into `core` or `LlmProviders` yet

# References

## Python

- `litellm/llms/deepseek/messages/transformation.py` (`DeepSeekAnthropicMessagesConfig`)

## Docs

- https://api-docs.deepseek.com/guides/anthropic_api/
