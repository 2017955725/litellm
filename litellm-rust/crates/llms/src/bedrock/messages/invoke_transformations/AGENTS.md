# Scope

- Bedrock InvokeModel adapter for native Anthropic Messages on `bedrock-runtime`, reached by `bedrock/<model>` and `bedrock/invoke/<model>`

# Invariants

- A forwarded caller `Authorization` header must never stand in for the Bedrock bearer token or for SigV4 signing
- Betas travel in the body as `anthropic_beta`, never as an `anthropic-beta` header, and only betas in the Bedrock beta mapping survive so unmapped caller betas don't cause a 400
- Top-level body fields outside the Bedrock InvokeModel allowlist are removed, and `stream` and `model` are dropped because the URL carries both
- Streaming holds each `message_delta` until the next event so cache usage from `message_stop` (or `message_start`) lands on it, otherwise cost logging sees zero or negative cache input

# Gotchas

- Anthropic's server-side `web_search_*` tools are rejected with a 400 unless web search interception already rewrote them
- In streaming, `amazon-bedrock-invocationMetrics` fills `usage`, but the chunk's own usage fields win over the metrics
- Errors map to Bedrock errors, not Anthropic ones

# Known gaps

- `transform_anthropic_messages_request` returns `Unsupported` and core calls it on every request, so every Bedrock Messages call fails until the Bedrock body shaping (version, betas, thinking injection, cache_control, output_config, tools, context_management, allowlist) is ported
- Core maps every Bedrock model to this adapter without Python's `claude` and `converse/` gate
- Region and credentials resolve from an empty param map, so `aws_region_name` and `aws_bedrock_runtime_endpoint` are ignored
- The `model_id` param override and URL-encoding of ARN model IDs (inference profiles, provisioned throughput) are missing
- The proxy-owned `X-Amzn-Bedrock-Request-Metadata` header from `bedrock_request_metadata_fields` is not added, and caller copies are not dropped

# References

## Python

- `litellm/llms/bedrock/messages/invoke_transformations/anthropic_claude3_transformation.py` (`AmazonAnthropicClaudeMessagesConfig`)
- `litellm/llms/bedrock/chat/invoke_transformations/base_invoke_transformation.py` (`AmazonInvokeConfig`, URL and model ID)
- `litellm/llms/bedrock/base_aws_llm.py` (`BaseAWSLLM`, signing)

## Docs

- [Claude Messages request and response](https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters-anthropic-claude-messages-request-response.html)
- [Claude Messages request and response, Markdown](https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters-anthropic-claude-messages-request-response.md)
- [InvokeModelWithResponseStream](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_runtime_InvokeModelWithResponseStream.html)
- [Bedrock user guide index, llms.txt](https://docs.aws.amazon.com/bedrock/latest/userguide/llms.txt)
