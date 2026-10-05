use serde_json::{Map, Value};

use crate::recognized::Recognized;
use crate::serde_compat::deserialize_present;

use super::CacheControl;
use crate::json_schema::JsonSchema;

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
#[derive(Default)]
pub struct ToolDefinition {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub name: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub description: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub input_schema: Option<Recognized<JsonSchema>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub strict: Option<Recognized<bool>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub cache_control: Option<Recognized<CacheControl>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub defer_loading: Option<Recognized<bool>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub allowed_callers: Option<Recognized<Vec<String>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub input_examples: Option<Recognized<Vec<Map<String, Value>>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub eager_input_streaming: Option<Recognized<bool>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub display_width_px: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub display_height_px: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub display_number: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub max_uses: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub max_tokens: Option<Recognized<u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub allowed_domains: Option<Recognized<Vec<String>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub blocked_domains: Option<Recognized<Vec<String>>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub citations: Option<Recognized<super::CitationsConfig>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub user_location: Option<Recognized<WebSearchUserLocation>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub model: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub caching: Option<Recognized<CacheControl>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct WebSearchUserLocation {
    #[serde(rename = "type")]
    pub location_type: UserLocationType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub city: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub country: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub region: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub timezone: Option<Recognized<String>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum UserLocationType {
    Approximate,
}

#[serde_with::skip_serializing_none]
#[macro_rules_attribute::apply(wire_type)]
pub struct ToolChoice {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceType,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub name: Option<Recognized<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    pub disable_parallel_tool_use: Option<Recognized<bool>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[macro_rules_attribute::apply(wire_type)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceType {
    Auto,
    Any,
    Tool,
    None,
}
