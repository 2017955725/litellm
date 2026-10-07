# Scope

- The `bedrock/mantle/<model>` route: the wire adapter for the bedrock-mantle endpoint's native Anthropic Messages API
- The `bedrock_mantle` provider reuses this adapter for its Claude models by composition from `bedrock_mantle/messages`. Everything that differs between the two spellings is settings and auth, never wire format

# Invariants

- The model goes in the body with one leading `mantle/` segment removed, unlike Invoke, which puts it in the URL
- `anthropic_version` and `anthropic_beta` never appear in the body; send them as headers, and drop `anthropic-beta` when no beta survives
- Streams are Anthropic SSE, not the AWS eventstream that Invoke returns
- Beta filtering happens once, in this adapter, against the mantle column of the Anthropic beta map

# Boundaries

- Credential and settings lookup for the `bedrock_mantle` spelling lives in `bedrock_mantle/messages`. This folder only reads the `bedrock` provider's `aws_*` settings
- Never import `bedrock_mantle` from here; the dependency only points from `bedrock_mantle` to `bedrock`

# Gotchas

- `BEDROCK_MANTLE_API_BASE` is shared with the OpenAI-compatible mantle routes, so an override can carry `/v1` or `/openai/v1`. Strip any known base suffix before appending the messages path exactly once
- When no API key resolves, SigV4 signs for service `bedrock` and the credential scope must use the region in the URL host, or a stale `aws_region_name` breaks the signature

# Decisions

- One adapter for both spellings. Python's two paths drifted (accepted `api_base` suffixes, and `clear_thinking_20251015` accepted only on `bedrock_mantle`). Rust implements the union for both instead of copying the drift

# References

## Python

- `litellm/llms/bedrock/messages/mantle_transformation.py`
- `build_mantle_messages_url` in `litellm/llms/bedrock/common_utils.py`

## Docs

- [Native Anthropic Messages API, including Mantle](https://docs.aws.amazon.com/bedrock/latest/userguide/inference-messages-api.md)
- [APIs supported by Amazon Bedrock, by endpoint](https://docs.aws.amazon.com/bedrock/latest/userguide/apis.md)
- [Bedrock documentation index, llms.txt](https://docs.aws.amazon.com/bedrock/latest/userguide/llms.txt)
