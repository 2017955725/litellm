# Scope

- Messages adapter for Claude Platform on AWS, the Anthropic-operated Messages API behind an AWS gateway, reached as `bedrock/claude_platform/<model>`

# Invariants

- The `claude_platform/` prefix selects this adapter before any converse or invoke check, with no base-model or capability gating, and is stripped once before the model goes on the wire
- A workspace ID is required, and a missing one is an authentication error raised before any request is sent
- An api key means `x-api-key` auth with no signing. Without one the request is SigV4 signed with the Bedrock credential chain, and the signature must cover the final serialized body
- Workspace and override keys, every `aws_*` key, and gateway-rejected params never reach the body
- Automatic betas are computed on the filtered body, so a dropped param never pulls in a beta

# Gotchas

- User `anthropic-beta` values are forwarded as is, unlike plain Bedrock, because this gateway serves the full Anthropic platform and Python turns beta filtering off here
- `claude_platform_unsupported_params` replaces the default rejected set (`context_management`) instead of extending it, and a non-list value is ignored with a warning
- Responses and streams are native Anthropic Messages with no Bedrock event-stream framing, and only error mapping reports them as Bedrock errors

# Decisions

- Anthropic payload, beta and response policy and Bedrock region validation and SigV4 signing are reused explicitly, so this folder only holds the workspace, URL, auth mode and body filtering differences

# Known gaps

- `mod.rs` is empty and nothing in Rust recognizes the `claude_platform/` route yet, so the whole adapter is still to be ported

# References

- `litellm/llms/bedrock/claude_platform/messages_transformation.py` (`BedrockClaudePlatformMessagesConfig`)
- `litellm/llms/bedrock/claude_platform/common_utils.py`
- `litellm/llms/bedrock/claude_platform/transformation.py`, the chat config, only as a cross-check of the same auth and body filtering
- [Claude Platform on AWS overview](https://platform.claude.com/docs/en/build-with-claude/claude-platform-on-aws) ([Markdown](https://platform.claude.com/docs/en/build-with-claude/claude-platform-on-aws.md), [llms.txt](https://platform.claude.com/llms.txt))
- [Making requests, AWS user guide](https://docs.aws.amazon.com/claude-platform/latest/userguide/making-requests.html) ([Markdown](https://docs.aws.amazon.com/claude-platform/latest/userguide/making-requests.md), [llms.txt](https://docs.aws.amazon.com/claude-platform/latest/userguide/llms.txt))
- [Messages API](https://platform.claude.com/docs/en/api/http/messages/create)
