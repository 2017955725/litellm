# Scope

- The generic Messages adapter for OpenAI-compatible servers that also serve the Anthropic Messages API natively
- Reached by a `providers.json` entry listing `/v1/messages` in `supported_endpoints`, only after every hand-written provider config declined, or by any deployment whose `model_info.supported_endpoints` contains `/v1/messages` when no provider config matched

# Invariants

- Never key behavior on a provider slug and never keep a list of slugs. Adding a provider is a JSON change, not new Rust code
- Do not filter `anthropic-beta` values against an Anthropic allowlist, since these servers decide for themselves which betas they accept
- Never override caller headers, and add bearer auth only when the caller sent neither `authorization` nor `x-api-key`
- Strip `ttl` from every `cache_control` unless the registry sets `constraints.cache_control_ttl` or the deployment sets `model_info.cache_control_ttl`, because strict servers reject the whole request over it

# Boundaries

- `providers.json` gets one registry under `openai_like` that parses the Python file and is shared by chat and messages. Never copy entries into Rust or keep a messages-only registry
- Responses and streams decode exactly like native Anthropic, so call the `anthropic/messages` helpers explicitly instead of adding a parser here. That reuse does not make Anthropic request policy a default here

# Gotchas

- Thinking does not follow Anthropic thinking semantics, so nothing is dropped or rewritten for it
- Registry providers strip `x-anthropic-billing-header` system blocks while the per-deployment base keeps them
- An api base already ending in `/v1/messages` is used as is, otherwise one trailing `/v1` is removed before appending `/v1/messages`

# Known gaps

- The Rust adapter does not exist yet: `mod.rs` is empty
- Rust does not load `providers.json` for chat or messages yet

# References

- `litellm/llms/openai_like/messages/transformation.py` (`OpenAILikeAnthropicMessagesConfig` for per-deployment, `JSONProviderAnthropicMessagesConfig` for registry providers)
- `litellm/llms/openai_like/providers.json` and `ProviderConfigManager._get_provider_anthropic_messages_config_cached` in `litellm/utils.py`
- [Anthropic Messages protocol reference for this generic adapter](https://platform.claude.com/docs/en/api/messages/create)
- [Anthropic documentation index, llms.txt](https://platform.claude.com/llms.txt)
