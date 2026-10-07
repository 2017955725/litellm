use litellm_auth::{
    CredentialPlacement, CredentialPlanKind, CredentialRule, ExistingHeaderBehavior,
    ProviderAuthPolicy, SecretValue,
};
use litellm_llms_types::formats::messages::MessagesRequest;
use serde_json::{Map, Value, json};

use crate::{
    Error, ErrorDetail,
    anthropic::{
        common_utils::DEFAULT_ANTHROPIC_HEADERS,
        messages::{
            handler::shape_anthropic_messages_request,
            transformation::{
                BillingMetadata, COMPATIBLE_HOST_REQUEST_POLICY, RequestPolicy,
                transform_messages_request_with, update_headers_with_anthropic_beta,
            },
        },
    },
    base_llm::{
        auth::{AuthScheme, Headers, ValidatedEnvironment},
        messages::{
            context::MessagesTransformContext,
            transformation::{BaseMessagesConfig, complete_messages_url},
        },
    },
    openai_like::common_utils::{JsonProvider, non_blank},
};

pub const OPENAI_LIKE_MESSAGES_AUTH_POLICY: ProviderAuthPolicy = ProviderAuthPolicy {
    rules: &[CredentialRule {
        kind: CredentialPlanKind::Static,
        placement: CredentialPlacement::Bearer,
    }],
    accepted_existing_headers: &["authorization", "x-api-key"],
    existing_header_behavior: ExistingHeaderBehavior::Preserve,
    scope: None,
    audience: None,
};

/// Python's `OpenAILikeAnthropicMessagesConfig` (`Deployment`, opted into through
/// `model_info.supported_endpoints`) and `JSONProviderAnthropicMessagesConfig` (`Registry`, a
/// `providers.json` entry listing `/v1/messages`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenAILikeMessagesConfig {
    Deployment { cache_control_ttl: bool },
    Registry(&'static JsonProvider),
}

pub const OPENAI_LIKE_MESSAGES_CONFIG: OpenAILikeMessagesConfig =
    OpenAILikeMessagesConfig::Deployment {
        cache_control_ttl: false,
    };

pub const OPENAI_LIKE_MESSAGES_CONFIG_WITH_CACHE_CONTROL_TTL: OpenAILikeMessagesConfig =
    OpenAILikeMessagesConfig::Deployment {
        cache_control_ttl: true,
    };

impl OpenAILikeMessagesConfig {
    pub fn request_policy(self) -> RequestPolicy {
        RequestPolicy {
            billing_metadata: match self {
                Self::Deployment { .. } => BillingMetadata::Forward,
                Self::Registry(_) => BillingMetadata::Strip,
            },
            ..COMPATIBLE_HOST_REQUEST_POLICY
        }
    }

    pub fn keeps_cache_control_ttl(self) -> bool {
        match self {
            Self::Deployment { cache_control_ttl } => cache_control_ttl,
            Self::Registry(provider) => provider.cache_control_ttl,
        }
    }

    pub fn resolve_api_key(
        self,
        api_key: Option<&str>,
        env_lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Option<String> {
        match self {
            Self::Deployment { .. } => non_blank(api_key).map(str::to_string),
            Self::Registry(provider) => provider.resolve_api_key(api_key, env_lookup),
        }
    }
}

impl BaseMessagesConfig for OpenAILikeMessagesConfig {
    fn shape_request(
        &self,
        request: MessagesRequest,
        reasoning_auto_summary: bool,
    ) -> Result<MessagesRequest, Error> {
        shape_anthropic_messages_request(request, reasoning_auto_summary)
    }

    fn get_complete_url(
        &self,
        api_base: Option<&str>,
        _model: &str,
        env_lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Result<String, Error> {
        match self {
            Self::Deployment { .. } => non_blank(api_base)
                .map(complete_messages_url)
                .ok_or(Error::MissingField("api_base")),
            Self::Registry(provider) => Ok(complete_messages_url(
                &provider.resolve_api_base(api_base, env_lookup),
            )),
        }
    }

    fn transform_anthropic_messages_request(
        &self,
        request: MessagesRequest,
        context: &MessagesTransformContext,
    ) -> Result<MessagesRequest, Error> {
        let request = transform_messages_request_with(request, context, self.request_policy())?;
        if self.keeps_cache_control_ttl() {
            return Ok(request);
        }
        with_portable_cache_control(request)
    }

    fn secret_names(&self) -> &'static [&'static str] {
        match self {
            Self::Deployment { .. } => &[],
            Self::Registry(provider) => provider.secret_names(),
        }
    }

    /// Keyless servers are valid, so a missing key sends no credential instead of failing.
    fn validate_environment(
        &self,
        headers: Headers,
        api_key: Option<&str>,
        _model: &str,
        env_lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Result<ValidatedEnvironment, Error> {
        let key = self.resolve_api_key(api_key, env_lookup);
        let auth = match key {
            Some(key) if !OPENAI_LIKE_MESSAGES_AUTH_POLICY.has_existing_credential(&headers) => {
                AuthScheme::Credential {
                    placement: CredentialPlacement::Bearer,
                    secret: SecretValue::new(key),
                }
            }
            _ => AuthScheme::Forwarded,
        };
        Ok(ValidatedEnvironment { headers, auth })
    }

    fn default_headers(&self) -> &'static [(&'static str, &'static str)] {
        DEFAULT_ANTHROPIC_HEADERS
    }

    fn request_headers(&self, headers: Headers, request: &MessagesRequest) -> Headers {
        update_headers_with_anthropic_beta(headers, request)
    }
}

/// Python's `normalize_cache_control_in_anthropic_payload`: every `cache_control` the Messages
/// API defines (request, system blocks, tools, message blocks, `tool_result` content) becomes
/// `{"type": <its type, or "ephemeral">}`, and a non-object one is dropped. Application data
/// such as `tool_use.input` and `input_schema` is never touched.
pub fn with_portable_cache_control(request: MessagesRequest) -> Result<MessagesRequest, Error> {
    let fields = match serde_json::to_value(request) {
        Ok(Value::Object(fields)) => fields,
        Ok(_) => {
            return Err(Error::InvalidType {
                expected: "an object",
                actual: "a non-object request",
            });
        }
        Err(error) => {
            return Err(Error::InvalidRequest(ErrorDetail::invalid(
                "request", error,
            )));
        }
    };
    let portable: Map<String, Value> = portable_block(fields)
        .into_iter()
        .map(|(key, value)| {
            let value = match key.as_str() {
                "system" | "tools" => portable_blocks(value),
                "messages" => portable_messages(value),
                _ => value,
            };
            (key, value)
        })
        .collect();
    serde_json::from_value(Value::Object(portable))
        .map_err(|error| Error::InvalidRequest(ErrorDetail::invalid("request", error)))
}

fn portable_block(block: Map<String, Value>) -> Map<String, Value> {
    if !block.contains_key("cache_control") {
        return block;
    }
    block
        .into_iter()
        .filter_map(|(key, value)| match key.as_str() {
            "cache_control" => portable_cache_control(&value).map(|value| (key, value)),
            _ => Some((key, value)),
        })
        .collect()
}

fn portable_cache_control(cache_control: &Value) -> Option<Value> {
    let cache_control = cache_control.as_object()?;
    let cache_type = cache_control
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("ephemeral");
    Some(json!({ "type": cache_type }))
}

fn portable_blocks(blocks: Value) -> Value {
    match blocks {
        Value::Array(blocks) => blocks
            .into_iter()
            .map(|block| match block {
                Value::Object(block) => Value::Object(portable_block(block)),
                other => other,
            })
            .collect(),
        other => other,
    }
}

fn portable_content_block(block: Value) -> Value {
    let Value::Object(block) = block else {
        return block;
    };
    let block = portable_block(block);
    if block.get("type").and_then(Value::as_str) != Some("tool_result") {
        return Value::Object(block);
    }
    block
        .into_iter()
        .map(|(key, value)| match key.as_str() {
            "content" => (key, portable_blocks(value)),
            _ => (key, value),
        })
        .collect()
}

fn portable_messages(messages: Value) -> Value {
    match messages {
        Value::Array(messages) => messages.into_iter().map(portable_message).collect(),
        other => other,
    }
}

fn portable_message(message: Value) -> Value {
    let Value::Object(message) = message else {
        return message;
    };
    message
        .into_iter()
        .map(|(key, value)| match (key.as_str(), value) {
            ("content", Value::Array(blocks)) => (
                key,
                blocks.into_iter().map(portable_content_block).collect(),
            ),
            (_, value) => (key, value),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use litellm_llms_types::formats::messages::MessagesResponse;
    use rstest::rstest;

    use super::*;

    static REGISTRY_PROVIDER: JsonProvider = JsonProvider::new(
        "https://registry.example/v1",
        "REGISTRY_API_KEY",
        Some("REGISTRY_API_BASE"),
        false,
    );
    static REGISTRY_PROVIDER_WITHOUT_BASE_ENV: JsonProvider =
        JsonProvider::new("https://keyonly.example", "KEYONLY_API_KEY", None, false);
    static REGISTRY_PROVIDER_WITH_TTL: JsonProvider = JsonProvider::new(
        "https://ttl.example/v1",
        "TTL_API_KEY",
        Some("TTL_API_BASE"),
        true,
    );

    const REGISTRY: OpenAILikeMessagesConfig =
        OpenAILikeMessagesConfig::Registry(&REGISTRY_PROVIDER);

    fn request_from(value: Value) -> MessagesRequest {
        serde_json::from_value(value).expect("valid request")
    }

    fn headers(pairs: &[(&str, &str)]) -> Headers {
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
    }

    fn transformed(config: OpenAILikeMessagesConfig, body: Value) -> Value {
        serde_json::to_value(
            config
                .transform_anthropic_messages_request(
                    request_from(body),
                    &MessagesTransformContext::default(),
                )
                .expect("request transforms"),
        )
        .expect("serializable request")
    }

    fn cache_controlled_body(cache_control: Value) -> Value {
        json!({
            "model": "m",
            "max_tokens": 16,
            "cache_control": cache_control,
            "system": [{"type": "text", "text": "sys", "cache_control": cache_control}],
            "tools": [{
                "name": "lookup",
                "input_schema": {"type": "object", "properties": {"cache_control": {"type": "string"}}},
                "cache_control": cache_control
            }],
            "messages": [
                {"role": "user", "content": [
                    {"type": "text", "text": "hi", "cache_control": cache_control},
                    {"type": "tool_result", "tool_use_id": "t1", "cache_control": cache_control, "content": [
                        {"type": "text", "text": "out", "cache_control": cache_control}
                    ]}
                ]},
                {"role": "assistant", "content": [
                    {"type": "tool_use", "id": "t2", "name": "lookup", "input": {"cache_control": {"ttl": "1h"}}}
                ]}
            ]
        })
    }

    fn cache_controls(body: &Value) -> Vec<Value> {
        [
            &body["cache_control"],
            &body["system"][0]["cache_control"],
            &body["tools"][0]["cache_control"],
            &body["messages"][0]["content"][0]["cache_control"],
            &body["messages"][0]["content"][1]["cache_control"],
            &body["messages"][0]["content"][1]["content"][0]["cache_control"],
        ]
        .into_iter()
        .cloned()
        .collect()
    }

    #[rstest]
    #[case::deployment_default(OPENAI_LIKE_MESSAGES_CONFIG)]
    #[case::registry_default(REGISTRY)]
    fn ttl_is_stripped_from_every_scoped_cache_control(#[case] config: OpenAILikeMessagesConfig) {
        let body = transformed(
            config,
            cache_controlled_body(json!({"type": "ephemeral", "ttl": "1h", "scope": "global"})),
        );
        assert_eq!(cache_controls(&body), vec![json!({"type": "ephemeral"}); 6]);
        assert_eq!(
            body["tools"][0]["input_schema"]["properties"]["cache_control"],
            json!({"type": "string"})
        );
        assert_eq!(
            body["messages"][1]["content"][0]["input"],
            json!({"cache_control": {"ttl": "1h"}})
        );
    }

    #[rstest]
    #[case::deployment_opt_in(OPENAI_LIKE_MESSAGES_CONFIG_WITH_CACHE_CONTROL_TTL)]
    #[case::registry_constraint(OpenAILikeMessagesConfig::Registry(&REGISTRY_PROVIDER_WITH_TTL))]
    fn ttl_is_kept_when_the_deployment_or_registry_allows_it(
        #[case] config: OpenAILikeMessagesConfig,
    ) {
        let cache_control = json!({"type": "ephemeral", "ttl": "1h"});
        let body = transformed(config, cache_controlled_body(cache_control.clone()));
        assert_eq!(cache_controls(&body), vec![cache_control; 6]);
    }

    #[test]
    fn a_cache_control_without_a_type_becomes_ephemeral_and_a_null_one_is_dropped() {
        let body = transformed(
            OPENAI_LIKE_MESSAGES_CONFIG,
            json!({
                "model": "m",
                "max_tokens": 16,
                "messages": [{"role": "user", "content": [
                    {"type": "text", "text": "a", "cache_control": {"ttl": "5m"}},
                    {"type": "text", "text": "b", "cache_control": null}
                ]}]
            }),
        );
        assert_eq!(
            body["messages"][0]["content"],
            json!([
                {"type": "text", "text": "a", "cache_control": {"type": "ephemeral"}},
                {"type": "text", "text": "b"}
            ])
        );
    }

    #[rstest]
    #[case::deployment_keeps_them(OPENAI_LIKE_MESSAGES_CONFIG, true)]
    #[case::registry_strips_them(REGISTRY, false)]
    fn billing_metadata_system_blocks(
        #[case] config: OpenAILikeMessagesConfig,
        #[case] kept: bool,
    ) {
        let billing = json!({"type": "text", "text": "x-anthropic-billing-header: cc_version=1"});
        let body = transformed(
            config,
            json!({
                "model": "m",
                "max_tokens": 16,
                "system": [billing.clone(), {"type": "text", "text": "be terse"}],
                "messages": [{"role": "user", "content": "hi"}]
            }),
        );
        assert_eq!(
            body["system"]
                .as_array()
                .expect("system blocks")
                .contains(&billing),
            kept
        );
        assert_eq!(
            body["system"].as_array().expect("system blocks").last(),
            Some(&json!({"type": "text", "text": "be terse"}))
        );
    }

    #[rstest]
    #[case::deployment(OPENAI_LIKE_MESSAGES_CONFIG)]
    #[case::registry(REGISTRY)]
    fn thinking_and_temperature_pass_through_untouched(#[case] config: OpenAILikeMessagesConfig) {
        for thinking in [
            json!({"type": "disabled"}),
            json!({"type": "enabled", "budget_tokens": 1024}),
            json!({"type": "adaptive"}),
        ] {
            let body = json!({
                "model": "m",
                "max_tokens": 4096,
                "temperature": 0.3,
                "thinking": thinking,
                "messages": [{"role": "user", "content": "hi"}]
            });
            assert_eq!(transformed(config, body.clone()), body);
        }
    }

    #[test]
    fn max_tokens_is_still_required() {
        assert_eq!(
            OPENAI_LIKE_MESSAGES_CONFIG.transform_anthropic_messages_request(
                request_from(
                    json!({"model": "m", "messages": [{"role": "user", "content": "hi"}]})
                ),
                &MessagesTransformContext::default(),
            ),
            Err(Error::MissingField("max_tokens"))
        );
    }

    #[rstest]
    #[case::bare_host("https://h.example", "https://h.example/v1/messages")]
    #[case::version_suffix("https://h.example/v1/", "https://h.example/v1/messages")]
    #[case::complete_endpoint("https://h.example/x/v1/messages", "https://h.example/x/v1/messages")]
    fn deployment_url_completes_the_given_base(#[case] base: &str, #[case] expected: &str) {
        assert_eq!(
            OPENAI_LIKE_MESSAGES_CONFIG.get_complete_url(Some(base), "m", &|_| None),
            Ok(expected.to_string())
        );
    }

    #[rstest]
    #[case::absent(None)]
    #[case::blank(Some(" "))]
    fn deployment_url_requires_an_api_base_and_ignores_env(#[case] base: Option<&str>) {
        assert_eq!(
            OPENAI_LIKE_MESSAGES_CONFIG
                .get_complete_url(base, "m", &|_| Some("https://env.example".to_string())),
            Err(Error::MissingField("api_base"))
        );
    }

    #[rstest]
    #[case::param_wins(
        Some("https://param.example"),
        Some("https://env.example"),
        "https://param.example/v1/messages"
    )]
    #[case::env_over_default(
        None,
        Some("https://env.example/v1"),
        "https://env.example/v1/messages"
    )]
    #[case::blank_param_falls_through(
        Some(""),
        Some("https://env.example"),
        "https://env.example/v1/messages"
    )]
    #[case::registry_default(None, None, "https://registry.example/v1/messages")]
    fn registry_url_resolves_param_then_env_then_default(
        #[case] param: Option<&str>,
        #[case] env: Option<&str>,
        #[case] expected: &str,
    ) {
        let lookup = |name: &str| {
            (name == "REGISTRY_API_BASE")
                .then_some(env)
                .flatten()
                .map(str::to_string)
        };
        assert_eq!(
            REGISTRY.get_complete_url(param, "m", &lookup),
            Ok(expected.to_string())
        );
    }

    fn auth_for(
        config: OpenAILikeMessagesConfig,
        forwarded: &[(&str, &str)],
        api_key: Option<&str>,
        env_lookup: &dyn Fn(&str) -> Option<String>,
    ) -> ValidatedEnvironment {
        config
            .validate_environment(headers(forwarded), api_key, "m", env_lookup)
            .expect("environment validates")
    }

    fn bearer_secret(validated: &ValidatedEnvironment) -> Option<String> {
        match &validated.auth {
            AuthScheme::Credential {
                placement: CredentialPlacement::Bearer,
                secret,
            } => Some(secret.expose().to_string()),
            _ => None,
        }
    }

    #[rstest]
    #[case::deployment(OPENAI_LIKE_MESSAGES_CONFIG)]
    #[case::registry(REGISTRY)]
    fn the_key_is_sent_as_a_bearer(#[case] config: OpenAILikeMessagesConfig) {
        let validated = auth_for(config, &[("x-trace", "1")], Some("sk-param"), &|_| None);
        assert_eq!(bearer_secret(&validated), Some("sk-param".to_string()));
        assert_eq!(validated.headers, headers(&[("x-trace", "1")]));
    }

    #[rstest]
    #[case::authorization(&[("Authorization", "Bearer caller")])]
    #[case::x_api_key(&[("X-Api-Key", "caller")])]
    fn a_caller_credential_is_never_overridden(#[case] forwarded: &[(&str, &str)]) {
        for config in [OPENAI_LIKE_MESSAGES_CONFIG, REGISTRY] {
            let validated = auth_for(config, forwarded, Some("sk-param"), &|_| None);
            assert!(matches!(validated.auth, AuthScheme::Forwarded));
            assert_eq!(validated.headers, headers(forwarded));
        }
    }

    #[rstest]
    #[case::deployment(OPENAI_LIKE_MESSAGES_CONFIG)]
    #[case::registry(REGISTRY)]
    fn a_keyless_server_gets_no_credential(#[case] config: OpenAILikeMessagesConfig) {
        for api_key in [None, Some("  ")] {
            let validated = auth_for(config, &[], api_key, &|_| None);
            assert!(matches!(validated.auth, AuthScheme::Forwarded));
            assert_eq!(validated.headers, Headers::new());
        }
    }

    #[test]
    fn only_the_registry_falls_back_to_its_key_env() {
        let lookup = |name: &str| (name == "REGISTRY_API_KEY").then(|| "sk-env".to_string());
        assert_eq!(
            bearer_secret(&auth_for(REGISTRY, &[], None, &lookup)),
            Some("sk-env".to_string())
        );
        assert_eq!(
            bearer_secret(&auth_for(REGISTRY, &[], Some("sk-param"), &lookup)),
            Some("sk-param".to_string())
        );
        assert_eq!(
            bearer_secret(&auth_for(
                OPENAI_LIKE_MESSAGES_CONFIG,
                &[],
                None,
                &|_| Some("sk-env".to_string())
            )),
            None
        );
    }

    #[test]
    fn default_headers_are_the_anthropic_defaults() {
        for config in [OPENAI_LIKE_MESSAGES_CONFIG, REGISTRY] {
            assert_eq!(config.default_headers(), DEFAULT_ANTHROPIC_HEADERS);
        }
    }

    #[test]
    fn betas_are_merged_but_never_filtered() {
        let request = request_from(json!({
            "model": "m",
            "max_tokens": 16,
            "context_management": {"edits": [{"type": "compact_20260112"}]},
            "messages": [{"role": "user", "content": "hi"}]
        }));
        for config in [OPENAI_LIKE_MESSAGES_CONFIG, REGISTRY] {
            assert_eq!(
                config.request_headers(
                    headers(&[("Anthropic-Beta", "server-only-beta-2099-01-01")]),
                    &request
                ),
                headers(&[(
                    "anthropic-beta",
                    "compact-2026-01-12,server-only-beta-2099-01-01"
                )])
            );
        }
    }

    #[test]
    fn responses_pass_through() {
        let response: MessagesResponse = serde_json::from_value(json!({
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "hello"}],
            "model": "m",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {"input_tokens": 1, "output_tokens": 2}
        }))
        .expect("valid response");
        for config in [OPENAI_LIKE_MESSAGES_CONFIG, REGISTRY] {
            assert_eq!(
                config.transform_anthropic_messages_response("m", response.clone()),
                Ok(response.clone())
            );
            assert!(config.stream_decoder().is_none());
        }
    }

    fn requested_env(config: OpenAILikeMessagesConfig) -> Vec<String> {
        let requested = RefCell::new(Vec::<String>::new());
        let record = |name: &str| -> Option<String> {
            requested.borrow_mut().push(name.to_string());
            None
        };
        let _ = config.validate_environment(Vec::new(), None, "m", &record);
        let _ = config.get_complete_url(None, "m", &record);
        requested.into_inner()
    }

    #[rstest]
    #[case::deployment(OPENAI_LIKE_MESSAGES_CONFIG)]
    #[case::registry(REGISTRY)]
    #[case::registry_without_base_env(OpenAILikeMessagesConfig::Registry(
        &REGISTRY_PROVIDER_WITHOUT_BASE_ENV
    ))]
    fn secret_names_are_exactly_the_env_lookups(#[case] config: OpenAILikeMessagesConfig) {
        let requested = requested_env(config);
        assert_eq!(
            requested,
            config
                .secret_names()
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
        );
    }
}
