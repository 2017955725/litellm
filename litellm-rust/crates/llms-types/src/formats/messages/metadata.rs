use serde_json::{Map, Value};

use crate::json_schema::JsonSchema;
use crate::recognized::Recognized;
use crate::serde_compat::deserialize_present;

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct MessagesMetadata {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub user_id: Option<Recognized<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct OutputFormat {
    #[serde(rename = "type")]
    pub format_type: OutputFormatType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub schema: Option<Recognized<JsonSchema>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub strict: Option<Recognized<bool>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormatType {
    JsonSchema,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct MessagesCompaction {
    #[serde(rename = "type")]
    pub compaction_type: CompactionType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub instructions: Option<Recognized<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum CompactionType {
    Summarize,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct MessagesContainer {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub id: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub expires_at: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub skills: Option<Recognized<Vec<Recognized<ContainerSkill>>>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct ContainerSkill {
    #[serde(rename = "type")]
    pub skill_type: SkillType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub skill_id: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub version: Option<Recognized<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum SkillType {
    Anthropic,
    Custom,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct McpServer {
    #[serde(rename = "type")]
    pub server_type: McpServerType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub url: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub name: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub authorization_token: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub tool_configuration: Option<Recognized<McpToolConfiguration>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum McpServerType {
    Url,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct McpToolConfiguration {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub allowed_tools: Option<Recognized<Vec<String>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub enabled: Option<Recognized<bool>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct StopDetails {
    #[serde(rename = "type")]
    pub detail_type: StopDetailsType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub category: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub explanation: Option<Recognized<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum StopDetailsType {
    Refusal,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct ContextManagementResponse {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub applied_edits: Option<Recognized<Vec<Recognized<AppliedEdit>>>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct AppliedEdit {
    #[serde(default, deserialize_with = "deserialize_present")]
    #[serde(rename = "type")]
    pub edit_type: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cleared_input_tokens: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cleared_tool_uses: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cleared_thinking_turns: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub summary_input_tokens: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub summary_output_tokens: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub error: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub warnings: Option<Recognized<Vec<String>>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct Safeguard {
    #[serde(rename = "type")]
    pub safeguard_type: String,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub classifier_context: Option<Recognized<Map<String, Value>>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    strum::AsRefStr,
    serde_with::DeserializeFromStr,
    serde_with::SerializeDisplay,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(from = "String", into = "String"))]
#[strum(serialize_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    #[strum(default, transparent)]
    Other(String),
}

impl MessageRole {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl From<String> for MessageRole {
    fn from(value: String) -> Self {
        value.parse().unwrap_or_else(|never| match never {})
    }
}

impl From<MessageRole> for String {
    fn from(value: MessageRole) -> Self {
        value.to_string()
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    strum::AsRefStr,
    serde_with::DeserializeFromStr,
    serde_with::SerializeDisplay,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(from = "String", into = "String"))]
#[strum(serialize_all = "snake_case")]
pub enum MessageType {
    Message,
    #[strum(default, transparent)]
    Other(String),
}

impl MessageType {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl From<String> for MessageType {
    fn from(value: String) -> Self {
        value.parse().unwrap_or_else(|never| match never {})
    }
}

impl From<MessageType> for String {
    fn from(value: MessageType) -> Self {
        value.to_string()
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    strum::AsRefStr,
    serde_with::DeserializeFromStr,
    serde_with::SerializeDisplay,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(from = "String", into = "String"))]
#[strum(serialize_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    Refusal,
    Compaction,
    PauseTurn,
    ModelContextWindowExceeded,
    #[strum(default, transparent)]
    Other(String),
}

impl StopReason {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl From<String> for StopReason {
    fn from(value: String) -> Self {
        value.parse().unwrap_or_else(|never| match never {})
    }
}

impl From<StopReason> for String {
    fn from(value: StopReason) -> Self {
        value.to_string()
    }
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(untagged)]
pub enum ContainerReference {
    Id(String),
    Parameters(Box<MessagesContainer>),
}
