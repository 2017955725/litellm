# Scope

- The `bedrock_mantle` provider's Messages config: a thin layer that supplies this provider's settings and auth to the shared adapter in `bedrock/messages/mantle`
- Only model names containing `claude` select it. The provider's other models serve OpenAI-compatible APIs and fall back to the chat-completion adapter

# Invariants

- Wire format, beta filtering and stream decoding come from `bedrock/messages/mantle`. Do not fork or override them here
- A bearer token wins when one resolves. Otherwise sign with SigV4, removing any caller `authorization` header first

# Boundaries

- Provider-wide mantle helpers (auth, region resolution) belong in `bedrock_mantle/` so the future chat and responses ports share them, not in this folder

# Decisions

- `bedrock_mantle` stays a separate LiteLLM provider because pricing, provider resolution and env var names key on the `bedrock_mantle/` prefix. It still gets no wire adapter of its own: the endpoint is the same one the `bedrock/mantle/` route calls
- Composition, not inheritance: this config wraps the `bedrock` adapter, and `bedrock` never depends on `bedrock_mantle`

# References

- Python: `litellm/llms/bedrock_mantle/messages/transformation.py`, `litellm/llms/bedrock_mantle/common_utils.py`
- [Native Anthropic Messages API, including Mantle](https://docs.aws.amazon.com/bedrock/latest/userguide/inference-messages-api.md)
