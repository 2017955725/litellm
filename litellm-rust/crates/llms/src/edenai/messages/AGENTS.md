# Scope

- Eden AI's Anthropic Messages adapter for the `edenai` provider. Eden serves the Messages API for its whole catalog, so the payload is forwarded untranslated

# Invariants

- No model gating: every `edenai` model reaches this adapter, unlike Bedrock Mantle, Vertex, Azure AI and GitHub Copilot, which only accept models containing `claude`
- The model name is sent as given once the `edenai/` route prefix is removed. Eden accepts bare names, `anthropic/<model>` and its `@edenai` alias, so do not rewrite or validate it
- A missing API key is an authentication error raised before any request is sent
- A caller-supplied `authorization` or `x-api-key` header wins over the resolved key
- `cache_control` keeps its `ttl` because Eden supports it, so the openai_like reduction to `{"type": ...}` must stay off
- Betas are merged into one `anthropic-beta` header but never filtered against a provider allowlist
- The Anthropic-only thinking rewrites do not run, because Eden is not treated as having Anthropic thinking semantics
- System blocks carrying `x-anthropic-billing-header` metadata are stripped before sending

# Boundaries

- The `cost` extraction and the auth error stay here. Keep `cost` out of the shared Messages response schema

# Gotchas

- The default base already ends in `/v3`, so the final URL is `/v3/v1/messages`. A base ending in `/v1` loses it before `/v1/messages` is appended, and one already ending in `/v1/messages` is used unchanged
- Eden returns a top-level `cost` next to the Anthropic body. When present it replaces the price map estimate as the call's response cost, and a missing or malformed value is ignored
- Streams carry no `cost` yet, so streamed spend falls back to the price map

# Known gaps

- `mod.rs` is empty: the adapter is not ported to Rust yet, so everything above exists only in Python

# References

## Python

- `litellm/llms/edenai/messages/transformation.py` (`EdenAIAnthropicMessagesConfig`)
- `litellm/llms/openai_like/messages/transformation.py` (`JSONProviderAnthropicMessagesConfig`)
- `litellm/llms/edenai/common_utils.py`

## Docs

- [Create Anthropic Message](https://www.edenai.co/docs/api-reference/anthropic-messages/create-anthropic-message)
- [Create Anthropic Message, Markdown](https://www.edenai.co/docs/api-reference/anthropic-messages/create-anthropic-message.md)
- [Eden AI documentation index, llms.txt](https://www.edenai.co/docs/llms.txt)
