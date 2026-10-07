# Scope

- GitHub Copilot's Messages adapter for the `github_copilot` provider: endpoint, Copilot token auth, Copilot headers and beta policy. Payload, response and SSE handling are Anthropic's

# Invariants

- Only `github_copilot` models whose lowercased name contains `claude` reach this adapter. Every other Copilot model falls back to the chat completions bridge, and that gate lives in provider selection, not shared defaults
- The caller's `api_base` and `api_key` are always ignored. The base comes from `endpoints.api` in the cached Copilot token, so the Copilot bearer token is never sent to a caller-chosen host
- `openai-intent`, `x-interaction-type` and `x-github-api-version` are forced over caller values. All other Copilot defaults only fill keys the caller did not set
- A failure to obtain the Copilot token surfaces as an authentication error for provider `github_copilot`

# Boundaries

- Reuse `anthropic/messages` payload and response policy explicitly and keep only endpoint, auth, header and beta differences here

# Gotchas

- `anthropic-beta` values are never filtered against the provider beta allowlist, because Copilot forwards Messages natively and has no entry there
- Copilot does not execute `web_search` tools, so the adapter reports web search as not handled natively and the interception path short circuits web search only requests
- `X-Initiator` and `Copilot-Vision-Request` belong to the Copilot chat config. The Python Messages path does not send them, so do not add them here without a parity reason
- Copilot publishes no full HTTP spec for its `/v1/messages` endpoint, so the Anthropic Messages API is the working contract

# Known gaps

- `messages/mod.rs` is empty: there is no Rust adapter yet, so none of the rules above are implemented

# References

- `litellm/llms/github_copilot/messages/transformation.py` (`GithubCopilotAnthropicMessagesConfig`)
- `litellm/llms/github_copilot/authenticator.py`
- `litellm/llms/github_copilot/common_utils.py`
- [Copilot SDK streaming events, which list `/v1/messages` as a usage endpoint](https://docs.github.com/en/copilot/how-tos/copilot-sdk/features/streaming-events)
- [The same page as Markdown](https://docs.github.com/api/article/body?pathname=/en/copilot/how-tos/copilot-sdk/features/streaming-events)
- [GitHub documentation index, llms.txt](https://docs.github.com/llms.txt)
- [Anthropic Messages API](https://platform.claude.com/docs/en/api/http/messages/create)
