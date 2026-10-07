# Scope

- Vertex AI's native Messages adapter for Claude partner models (`vertex_ai/<claude model>`): endpoint, GCP auth policy, beta selection and Vertex request shaping

# Invariants

- Only `vertex_ai` models whose lowercased name contains `claude` use this adapter. Every other Vertex model goes through the chat-completion adapter, so do not widen the gate
- The location is untrusted input that ends up in a hostname, so validate it before building the URL: `global`, or a lowercase token of letters, digits and hyphens
- Resolve project, location and credentials without consuming them from the caller's params
- Copy headers before changing them so shared deployment headers stay untouched
- `model` is removed from the body because Vertex takes it only from the URL
- The URL is fixed during environment validation, and later URL resolution only returns it

# Boundaries

- Token minting and refresh belong in the `auth-gcp` crate. This folder only sends the resulting bearer token
- Responses and streams are Anthropic wire format and decode through `anthropic/messages` with no Vertex rewriting

# Gotchas

- The model catalog's `supported_regions` overrides the location: an unset location takes the first entry and an unsupported one is rerouted to it
- A location without a hyphen, such as `us` or `eu`, is a multi-region geography with its own `rep` host, not a malformed region
- Tool search tools add Vertex's own tool search beta, not the first-party one
- `scope` is stripped from every `cache_control` and billing metadata system blocks are always stripped
- `output_config.effort` is dropped for models that do not accept effort on Vertex

# Known gaps

- `mod.rs` is empty: the adapter is not implemented in Rust yet, so every rule above describes the target behavior from Python

# References

## Python

- `litellm/llms/vertex_ai/vertex_ai_partner_models/anthropic/experimental_pass_through/transformation.py` (`VertexAIPartnerModelsAnthropicMessagesConfig`)
- `litellm/llms/vertex_ai/vertex_llm_base.py` (`VertexBase`)
- `litellm/llms/vertex_ai/common_utils.py` (`get_vertex_base_url`)
- `litellm/llms/vertex_ai/vertex_ai_partner_models/anthropic/output_params_utils.py` (`sanitize_vertex_anthropic_output_params`)
- `litellm/utils.py` (`ProviderConfigManager._get_provider_anthropic_messages_config_cached`, adapter selection)

## Docs

- https://docs.cloud.google.com/gemini-enterprise-agent-platform/models/partner-models/claude/use-claude
- https://platform.claude.com/docs/en/build-with-claude/claude-on-vertex-ai.md
- https://platform.claude.com/docs/en/api/http/messages/create
