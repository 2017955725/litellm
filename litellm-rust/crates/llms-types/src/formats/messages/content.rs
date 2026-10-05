use serde_json::{Map, Value};

use crate::recognized::Recognized;
use crate::serde_compat::deserialize_present;

use crate::serde_compat::Nullable;

#[macro_rules_attribute::apply(wire_type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentSource {
    Base64 {
        media_type: String,
        data: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Url {
        url: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    File {
        file_id: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Text {
        media_type: String,
        data: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    Content {
        content: Recognized<BlockContent>,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(untagged)]
pub enum BlockContent {
    Text(String),
    Blocks(Vec<Recognized<ContentBlock>>),
    SearchError(WebSearchResultError),
    Block(Box<ContentBlock>),
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCaller {
    Direct {
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
    #[serde(rename = "code_execution_20250825")]
    CodeExecution {
        tool_id: String,
        #[serde(flatten)]
        extra: Map<String, Value>,
    },
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct CitationsConfig {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub enabled: Option<Recognized<bool>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct PageCitation {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cited_text: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub document_index: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub document_title: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub start_page_number: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub end_page_number: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct CharCitation {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cited_text: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub document_index: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub document_title: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub start_char_index: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub end_char_index: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct WebSearchCitation {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cited_text: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub url: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub title: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub encrypted_index: Option<Recognized<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Citation {
    PageLocation(PageCitation),
    CharLocation(CharCitation),
    WebSearchResultLocation(WebSearchCitation),
    ContentBlockLocation(ContentBlockCitation),
    SearchResultLocation(SearchResultCitation),
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(untagged)]
pub enum Citations {
    Config(CitationsConfig),
    Results(Vec<Recognized<Citation>>),
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct WebSearchResultError {
    #[serde(rename = "type")]
    pub error_type: WebSearchResultErrorType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub error_code: Option<Recognized<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchResultErrorType {
    WebSearchToolResultError,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct PromptCacheBreakpoint {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub mode: Option<Recognized<PromptCacheMode>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum PromptCacheMode {
    Explicit,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct ContentBlockCitation {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cited_text: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub document_index: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub document_title: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub start_block_index: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub end_block_index: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct SearchResultCitation {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cited_text: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub search_result_index: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub title: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub source: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub start_block_index: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub end_block_index: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    serde_with::DeserializeFromStr,
    serde_with::SerializeDisplay,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(from = "String", into = "String"))]
#[strum(serialize_all = "snake_case")]
pub enum ContentBlockType {
    Text,
    Image,
    Document,
    ContainerUpload,
    ToolReference,
    SearchResult,
    WebSearchResult,
    WebFetchResult,
    WebFetchToolResult,
    WebFetchToolResultError,
    CodeExecutionToolResult,
    CodeExecutionToolResultError,
    CodeExecutionResult,
    CodeExecutionOutput,
    EncryptedCodeExecutionResult,
    BashCodeExecutionToolResult,
    BashCodeExecutionToolResultError,
    BashCodeExecutionResult,
    BashCodeExecutionOutput,
    TextEditorCodeExecutionToolResult,
    TextEditorCodeExecutionToolResultError,
    TextEditorCodeExecutionViewResult,
    TextEditorCodeExecutionCreateResult,
    TextEditorCodeExecutionStrReplaceResult,
    ToolSearchToolResult,
    ToolSearchToolResultError,
    ToolSearchToolSearchResult,
    McpToolUse,
    McpToolResult,
    AdvisorResult,
    AdvisorRedactedResult,
    WebSearchToolResultError,
    Thinking,
    RedactedThinking,
    ToolUse,
    ServerToolUse,
    ToolResult,
    Compaction,
    AdvisorToolResult,
    WebSearchToolResult,
    #[strum(default, transparent)]
    Other(String),
}

impl From<String> for ContentBlockType {
    fn from(value: String) -> Self {
        value.parse().unwrap_or_else(|never| match never {})
    }
}

impl From<ContentBlockType> for String {
    fn from(value: ContentBlockType) -> Self {
        value.to_string()
    }
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct ContentBlock {
    #[serde(rename = "type", default, deserialize_with = "deserialize_present")]
    pub block_type: Option<Nullable<ContentBlockType>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub text: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub thinking: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub signature: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub data: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub id: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub name: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub tool_use_id: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub input: Option<Recognized<Map<String, Value>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub content: Option<Recognized<BlockContent>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub provider_specific_fields: Option<Recognized<Map<String, Value>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cache_control: Option<Nullable<CacheControl>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub source: Option<Recognized<ContentSource>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub citations: Option<Recognized<Citations>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub caller: Option<Recognized<ToolCaller>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub is_error: Option<Recognized<bool>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub file_id: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub title: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub context: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub tool_name: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub url: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub page_age: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub encrypted_content: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub snippet: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub prompt_cache_breakpoint: Option<Recognized<PromptCacheBreakpoint>>,
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

impl ContentBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            block_type: Some(Nullable::Value(ContentBlockType::Text)),
            text: Some(Nullable::Value(text.into())),
            ..Self::default()
        }
    }

    pub fn is_type(&self, block_type: ContentBlockType) -> bool {
        self.block_type.as_ref().and_then(Nullable::value) == Some(&block_type)
    }
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct CacheControl {
    #[serde(rename = "type", default, deserialize_with = "deserialize_present")]
    pub cache_type: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub ttl: Option<Nullable<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub scope: Option<Nullable<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
