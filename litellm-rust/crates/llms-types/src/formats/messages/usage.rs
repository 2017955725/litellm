use serde_json::{Map, Value};

use crate::recognized::Recognized;
use crate::serde_compat::Nullable;
use crate::serde_compat::deserialize_present;

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct ServerToolUsage {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub web_search_requests: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub web_fetch_requests: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct UsageIteration {
    #[serde(rename = "type")]
    pub iteration_type: UsageIterationType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub input_tokens: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub output_tokens: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum UsageIterationType {
    Compaction,
    Message,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct MessagesUsage {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub input_tokens: Option<Nullable<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub output_tokens: Option<Nullable<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cache_creation_input_tokens: Option<Nullable<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cache_read_input_tokens: Option<Nullable<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub server_tool_use: Option<Recognized<ServerToolUsage>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cache_creation: Option<Recognized<CacheCreationUsage>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub output_tokens_details: Option<Recognized<MessagesOutputTokensDetails>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub service_tier: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub speed: Option<Recognized<super::Speed>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub iterations: Option<Recognized<Vec<Recognized<UsageIteration>>>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct CacheCreationUsage {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub ephemeral_1h_input_tokens: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub ephemeral_5m_input_tokens: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct MessagesOutputTokensDetails {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub thinking_tokens: Option<Recognized<u64>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
