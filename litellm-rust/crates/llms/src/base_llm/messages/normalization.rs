use litellm_llms_types::{
    formats::messages::{
        CacheControl, ContentBlock, ContentBlockType, Message, MessageContent,
        MessagesOptionalParams, MessagesRequest, SystemPrompt,
    },
    recognized::Recognized,
    serde_compat::Nullable,
};

const SYSTEM_ROLE: &str = "system";
const BILLING_HEADER_PREFIX: &str = "x-anthropic-billing-header:";

fn content_into_blocks(content: MessageContent) -> Vec<ContentBlock> {
    match content {
        MessageContent::Text(text) => vec![ContentBlock::text(text)],
        MessageContent::Blocks(blocks) => blocks,
    }
}

fn system_into_blocks(system: Option<SystemPrompt>) -> Vec<ContentBlock> {
    match system {
        None => Vec::new(),
        Some(SystemPrompt::Text(text)) => vec![ContentBlock::text(text)],
        Some(SystemPrompt::Blocks(blocks)) => blocks,
    }
}

pub fn fold_system_role_messages(request: MessagesRequest) -> MessagesRequest {
    if !request
        .messages
        .iter()
        .any(|msg| msg.role.as_str() == SYSTEM_ROLE)
    {
        return request;
    }

    let (system_messages, chat_messages): (Vec<Message>, Vec<Message>) = request
        .messages
        .into_iter()
        .partition(|msg| msg.role.as_str() == SYSTEM_ROLE);

    let folded_system: Vec<ContentBlock> = system_into_blocks(request.params.system)
        .into_iter()
        .chain(
            system_messages
                .into_iter()
                .flat_map(|msg| content_into_blocks(msg.content)),
        )
        .collect();

    MessagesRequest {
        messages: chat_messages,
        params: MessagesOptionalParams {
            system: (!folded_system.is_empty()).then_some(SystemPrompt::Blocks(folded_system)),
            ..request.params
        },
        ..request
    }
}

fn is_billing_metadata(text: &str) -> bool {
    text.starts_with(BILLING_HEADER_PREFIX)
}

fn is_billing_metadata_block(block: &ContentBlock) -> bool {
    matches!(
        block.block_type,
        Some(Nullable::Value(ContentBlockType::Text))
    ) && matches!(&block.text, Some(Nullable::Value(text)) if is_billing_metadata(text))
}

fn without_billing_metadata(system: SystemPrompt) -> Option<SystemPrompt> {
    match system {
        SystemPrompt::Text(text) if text.is_empty() || is_billing_metadata(&text) => None,
        SystemPrompt::Text(text) => Some(SystemPrompt::Text(text)),
        SystemPrompt::Blocks(blocks) => {
            let kept: Vec<ContentBlock> = blocks
                .into_iter()
                .filter(|block| !is_billing_metadata_block(block))
                .collect();
            (!kept.is_empty()).then_some(SystemPrompt::Blocks(kept))
        }
    }
}

/// Drops the `x-anthropic-billing-header` text Claude Code puts in `system`, which only the
/// first-party API understands, and omits `system` when nothing else is left.
pub fn strip_billing_metadata(request: MessagesRequest) -> MessagesRequest {
    MessagesRequest {
        params: MessagesOptionalParams {
            system: request.params.system.and_then(without_billing_metadata),
            ..request.params
        },
        ..request
    }
}

fn without_scope(cache_control: CacheControl) -> CacheControl {
    CacheControl {
        scope: None,
        ..cache_control
    }
}

fn block_without_scope(block: ContentBlock) -> ContentBlock {
    ContentBlock {
        cache_control: block
            .cache_control
            .map(|cache_control| match cache_control {
                Nullable::Value(cache_control) => Nullable::Value(without_scope(cache_control)),
                other => other,
            }),
        ..block
    }
}

pub(crate) fn map_request_blocks(
    request: MessagesRequest,
    system_block: fn(ContentBlock) -> ContentBlock,
    message_block: fn(ContentBlock) -> ContentBlock,
) -> MessagesRequest {
    MessagesRequest {
        messages: request
            .messages
            .into_iter()
            .map(|message| Message {
                content: match message.content {
                    MessageContent::Blocks(blocks) => {
                        MessageContent::Blocks(blocks.into_iter().map(message_block).collect())
                    }
                    text => text,
                },
                ..message
            })
            .collect(),
        params: MessagesOptionalParams {
            system: request.params.system.map(|system| match system {
                SystemPrompt::Blocks(blocks) => {
                    SystemPrompt::Blocks(blocks.into_iter().map(system_block).collect())
                }
                text => text,
            }),
            ..request.params
        },
        ..request
    }
}

/// Removes `cache_control.scope`, which hosts other than the first-party API reject, from
/// the top-level `cache_control` and every `system` and message block.
pub fn strip_cache_control_scope(request: MessagesRequest) -> MessagesRequest {
    let request = map_request_blocks(request, block_without_scope, block_without_scope);
    MessagesRequest {
        params: MessagesOptionalParams {
            cache_control: request
                .params
                .cache_control
                .map(|cache_control| match cache_control {
                    Recognized::Known(cache_control) => {
                        Recognized::Known(without_scope(cache_control))
                    }
                    other => other,
                }),
            ..request.params
        },
        ..request
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::{Value, json};

    use super::*;

    fn system_after_strip(system: Value) -> Value {
        let request: MessagesRequest = serde_json::from_value(json!({
            "model": "m",
            "max_tokens": 1,
            "system": system,
            "messages": [{"role": "user", "content": "hi"}]
        }))
        .unwrap();
        serde_json::to_value(strip_billing_metadata(request)).unwrap()["system"].clone()
    }

    #[test]
    fn cache_control_scope_is_stripped_everywhere_and_idempotently() {
        let request: MessagesRequest = serde_json::from_value(json!({
            "model": "m",
            "max_tokens": 1,
            "cache_control": {"type": "ephemeral", "scope": "global"},
            "system": [{"type": "text", "text": "s", "cache_control": {"type": "ephemeral", "ttl": "1h", "scope": "global"}}],
            "messages": [
                {"role": "user", "content": [
                    {"type": "text", "text": "a", "cache_control": {"type": "ephemeral", "scope": "global"}},
                    {"type": "text", "text": "b"}
                ]},
                {"role": "assistant", "content": "plain"}
            ]
        }))
        .unwrap();
        let once = strip_cache_control_scope(request);
        let value = serde_json::to_value(&once).unwrap();
        assert_eq!(value["cache_control"], json!({"type": "ephemeral"}));
        assert_eq!(
            value["system"][0]["cache_control"],
            json!({"type": "ephemeral", "ttl": "1h"})
        );
        assert_eq!(
            value["messages"][0]["content"][0]["cache_control"],
            json!({"type": "ephemeral"})
        );
        assert_eq!(
            value["messages"][0]["content"][1],
            json!({"type": "text", "text": "b"})
        );
        assert_eq!(value["messages"][1]["content"], json!("plain"));
        assert_eq!(strip_cache_control_scope(once.clone()), once);
    }

    #[rstest]
    #[case::tool_definition(json!({"tools": [{
        "name": "lookup", "cache_control": {"type": "ephemeral", "scope": "global"}
    }]}))]
    #[case::nested_tool_result(json!({"messages": [{"role": "user", "content": [{
        "type": "tool_result", "content": [{"type": "text", "text": "result", "cache_control": {
            "type": "ephemeral", "scope": "global"
        }}]
    }]}]}))]
    #[case::application_input(json!({"messages": [{"role": "assistant", "content": [{
        "type": "tool_use", "input": {"cache_control": {"scope": "application-data"}}
    }]}]}))]
    #[case::unrecognized_top_level_cache(json!({"cache_control": {
        "type": false, "scope": "global"
    }}))]
    fn scope_stripping_preserves_sites_outside_its_existing_contract(#[case] fields: Value) {
        let Value::Object(fields) = fields else {
            panic!("case fields are an object")
        };
        let request: MessagesRequest = serde_json::from_value(Value::Object(
            [
                ("model".to_string(), json!("m")),
                ("messages".to_string(), json!([])),
            ]
            .into_iter()
            .chain(fields)
            .collect(),
        ))
        .unwrap();
        assert_eq!(strip_cache_control_scope(request.clone()), request);
    }

    #[rstest]
    #[case::billing_string(json!("x-anthropic-billing-header: cc_version=1"), Value::Null)]
    #[case::empty_string(json!(""), Value::Null)]
    #[case::plain_string(json!("be terse"), json!("be terse"))]
    #[case::only_billing_blocks(
        json!([{"type": "text", "text": "x-anthropic-billing-header: cc_version=1"}]),
        Value::Null
    )]
    #[case::billing_block_among_others(
        json!([
            {"type": "text", "text": "x-anthropic-billing-header: cc_version=1"},
            {"type": "text", "text": "be terse", "cache_control": {"type": "ephemeral"}}
        ]),
        json!([{"type": "text", "text": "be terse", "cache_control": {"type": "ephemeral"}}])
    )]
    #[case::prefix_not_at_start(
        json!([{"type": "text", "text": "see x-anthropic-billing-header: here"}]),
        json!([{"type": "text", "text": "see x-anthropic-billing-header: here"}])
    )]
    fn billing_metadata_is_stripped_from_system(#[case] system: Value, #[case] expected: Value) {
        assert_eq!(system_after_strip(system), expected);
    }
}
