use serde_json::{Map, Value};

use super::{
    Citation, ContentBlock, ContextManagementResponse, MessagesContainer, MessagesUsage,
    StopDetails,
};
use crate::recognized::Recognized;
use crate::serde_compat::{Nullable, deserialize_present};

#[macro_rules_attribute::apply(wire_type)]
pub struct MessagesStreamMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: super::MessageType,
    pub role: super::MessageRole,
    pub model: String,
    pub content: Vec<Recognized<ContentBlock>>,
    pub stop_reason: Option<super::StopReason>,
    pub stop_sequence: Option<String>,
    pub usage: MessagesUsage,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub safeguard_results: Option<Recognized<Vec<Map<String, Value>>>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessagesContentBlockDelta {
    TextDelta {
        text: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    InputJsonDelta {
        partial_json: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    #[serde(rename = "citations_delta")]
    Citations {
        citation: Box<Recognized<Citation>>,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    ThinkingDelta {
        thinking: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    SignatureDelta {
        signature: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    CompactionDelta {
        content: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct MessagesContentBlock {
    #[serde(rename = "type")]
    pub block_type: super::ContentBlockType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub id: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub name: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub text: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub input: Option<Recognized<Map<String, Value>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub thinking: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub signature: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub data: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub content: Option<Recognized<super::BlockContent>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub caller: Option<Recognized<super::ToolCaller>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub source: Option<Recognized<super::ContentSource>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub citations: Option<Recognized<super::Citations>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub tool_use_id: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub is_error: Option<Recognized<bool>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cache_control: Option<Recognized<super::CacheControl>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub file_id: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub title: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub context: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub url: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub page_age: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub encrypted_content: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub snippet: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub tool_name: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub provider_specific_fields: Option<Recognized<Map<String, Value>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub prompt_cache_breakpoint: Option<Recognized<super::PromptCacheBreakpoint>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub stdout: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub stderr: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub return_code: Option<Recognized<i64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub encrypted_stdout: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub error_code: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub error_message: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub retrieved_at: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub server_name: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub tool_references: Option<Recognized<Vec<Recognized<super::ContentBlock>>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub file_type: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub num_lines: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub start_line: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub total_lines: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub is_file_update: Option<Recognized<bool>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub lines: Option<Recognized<Vec<String>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub new_lines: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub new_start: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub old_lines: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub old_start: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct MessagesDelta {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub stop_reason: Option<Nullable<super::StopReason>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub stop_sequence: Option<Nullable<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub stop_details: Option<Recognized<StopDetails>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub container: Option<Recognized<MessagesContainer>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub safeguard_results: Option<Recognized<Vec<Map<String, Value>>>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
pub struct MessagesStreamError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub details: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessagesStreamEvent {
    MessageStart {
        message: Box<MessagesStreamMessage>,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    ContentBlockStart {
        index: u64,
        content_block: Box<MessagesContentBlock>,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    ContentBlockDelta {
        index: u64,
        delta: MessagesContentBlockDelta,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    ContentBlockStop {
        index: u64,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    MessageDelta {
        delta: Box<MessagesDelta>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage: Option<Box<MessagesUsage>>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "deserialize_present"
        )]
        context_management: Option<Box<Recognized<ContextManagementResponse>>>,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    MessageStop {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage: Option<Box<MessagesUsage>>,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Ping {
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Error {
        error: MessagesStreamError,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
}
