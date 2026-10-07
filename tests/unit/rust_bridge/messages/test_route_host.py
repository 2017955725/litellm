from types import MappingProxyType
from typing import Final

from litellm.rust_bridge.messages.route_host import arguments, response
from litellm.rust_bridge.messages.entrypoints import LiteLLMMessagesRequest
from dataclasses import astuple
import pytest
import litellm
from litellm.rust_bridge.messages import route_host


def test_response_is_a_detached_public_messages_dict() -> None:
    native: Final = MappingProxyType(
        {
            "id": "msg_native",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-5",
            "content": [{"type": "text", "text": "native"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 2, "output_tokens": 3},
        }
    )

    built: Final = response(native)

    assert built == dict(native)
    assert isinstance(built, dict)
    built["_hidden_params"] = {"annotated": True}
    assert "_hidden_params" not in native


def test_arguments_are_the_public_kwargs_view() -> None:
    kwargs: Final = MappingProxyType({"litellm_metadata": {"user_id": "u"}})
    request: Final = LiteLLMMessagesRequest(
        model="claude-sonnet-4-5",
        messages=[{"role": "user", "content": "hi"}],
        max_tokens=16,
        stream=None,
        api_key=None,
        api_base=None,
        custom_llm_provider="anthropic",
        kwargs=kwargs,
    )

    assert arguments(request) is kwargs


pytestmark = pytest.mark.usefixtures("local_model_cost_map")


def _flag_model(monkeypatch: pytest.MonkeyPatch, name: str, **flags: bool) -> None:
    monkeypatch.setitem(
        litellm.model_cost,
        name,
        {
            "litellm_provider": "anthropic",
            "mode": "chat",
            "input_cost_per_token": 0,
            "output_cost_per_token": 0,
            **flags,
        },
    )


def test_capabilities_come_from_the_model_map_under_the_callers_provider(monkeypatch: pytest.MonkeyPatch) -> None:
    _flag_model(
        monkeypatch,
        "claude-test-adaptive",
        supports_reasoning=True,
        supports_adaptive_thinking=True,
        supports_output_config=True,
        supports_xhigh_reasoning_effort=True,
        supports_sampling_params=False,
        supports_mid_conversation_system=True,
    )

    capabilities: Final = route_host.model_capabilities("anthropic/claude-test-adaptive", None)

    assert capabilities.supports_adaptive_thinking
    assert capabilities.supports_output_config
    assert not capabilities.supports_legacy_thinking
    assert not capabilities.supports_sampling_params
    assert capabilities.effort_tiers.xhigh
    assert not capabilities.effort_tiers.max
    assert capabilities.supports_mid_conversation_system


def test_bedrock_projection_carries_provider_specific_capabilities(monkeypatch: pytest.MonkeyPatch) -> None:
    model: Final = "anthropic.claude-test-projection"
    monkeypatch.setitem(
        litellm.model_cost,
        model,
        {
            "litellm_provider": "bedrock",
            "mode": "chat",
            "supports_native_structured_output": True,
            "supports_tool_search": True,
            "bedrock_output_config_effort_ceiling": "high",
        },
    )
    projected: Final = route_host.shaping(model, "bedrock", {})["capabilities"]
    assert projected["supports_native_structured_output"] is True
    assert projected["supports_tool_search"] is True
    assert projected["effort_ceiling"] == "high"

    other_provider: Final = route_host.shaping(model, "anthropic", {})["capabilities"]
    assert other_provider["supports_native_structured_output"] is False
    assert other_provider["supports_tool_search"] is False
    assert other_provider["effort_ceiling"] is None


@pytest.mark.parametrize("metadata_key", ["metadata", "litellm_metadata"])
def test_bedrock_metadata_projection_validates_text_without_resolving_policy(
    metadata_key: str, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(litellm, "bedrock_request_metadata_fields", ["user_api_key_alias", "spend_logs_metadata"])
    projected: Final = route_host.shaping(
        "bedrock/anthropic.claude-test",
        "bedrock",
        {
            metadata_key: {
                "user_api_key_alias": "prod-key",
                "user_api_key_spend": 1.25,
                123: "ignored",
                "spend_logs_metadata": {"cost_center": "cc-1", "invalid": 4, 456: "ignored"},
            }
        },
    )["bedrock_request_metadata"]
    assert projected == {
        "allowed_fields": ("user_api_key_alias", "spend_logs_metadata"),
        "sources": tuple(
            {
                "identity": (("user_api_key_alias", "prod-key"),) if name == metadata_key else (),
                "spend_logs": (("cost_center", "cc-1"),) if name == metadata_key else (),
            }
            for name in ("metadata", "litellm_metadata")
        ),
    }


@pytest.mark.parametrize("setting", [None, []])
def test_disabled_bedrock_metadata_projection_omits_identity(
    setting: list[str] | None, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(litellm, "bedrock_request_metadata_fields", setting)
    assert (
        route_host.shaping("bedrock/anthropic.claude-test", "bedrock", {"metadata": {"user_api_key_alias": "secret"}})[
            "bedrock_request_metadata"
        ]
        is None
    )


def test_bedrock_metadata_is_not_projected_for_another_provider(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(litellm, "bedrock_request_metadata_fields", ["user_api_key_alias"])
    assert (
        route_host.shaping("claude-test", "anthropic", {"metadata": {"user_api_key_alias": "secret"}})[
            "bedrock_request_metadata"
        ]
        is None
    )


def test_bedrock_connection_projection_preserves_typed_per_call_inputs() -> None:
    projected: Final = route_host.shaping(
        "bedrock/anthropic.claude-test",
        "bedrock",
        {
            "aws_bedrock_runtime_endpoint": "https://runtime.test",
            "aws_region_name": "us-east-2",
            "model_id": "override/model",
            "aws_bedrock_project_id": "project-test",
        },
    )["bedrock_connection"]
    assert projected == {
        "api_base": "https://runtime.test",
        "region": "us-east-2",
        "model_id": "override/model",
        "workspace_id": "project-test",
    }


def test_bedrock_connection_projection_drops_non_text_inputs() -> None:
    projected: Final = route_host.shaping(
        "bedrock/anthropic.claude-test", "bedrock", {"aws_region_name": 3, "model_id": {"unexpected": "value"}}
    )["bedrock_connection"]
    assert projected == {"api_base": None, "region": None, "model_id": None, "workspace_id": None}


def test_unmapped_model_keeps_sampling_params_and_no_reasoning_features() -> None:
    capabilities: Final = route_host.model_capabilities("anthropic/not-a-real-model", None)

    assert capabilities.supports_sampling_params
    assert not capabilities.supports_reasoning
    assert not capabilities.supports_adaptive_thinking
    assert not any(astuple(capabilities.effort_tiers))


@pytest.mark.parametrize(
    ("global_flag", "kwargs", "expected"),
    [
        (False, {}, False),
        (True, {}, True),
        (False, {"drop_params": "true"}, True),
        (False, {"drop_params": "nonsense"}, False),
        (False, {"drop_params": False}, False),
    ],
)
def test_drop_params_merges_the_global_flag_with_the_request(
    monkeypatch: pytest.MonkeyPatch, global_flag: bool, kwargs: dict[str, object], expected: bool
) -> None:
    monkeypatch.setattr(litellm, "drop_params", global_flag)

    assert route_host.shaping("anthropic/not-a-real-model", None, kwargs)["drop_params"] is expected


@pytest.mark.parametrize(
    ("configured", "expected"),
    [
        (["tools[*].input_examples", 3, "metadata.user_id"], ("tools[*].input_examples", "metadata.user_id")),
        ("tools", ()),
        (None, ()),
    ],
)
def test_additional_drop_params_keep_only_string_paths(configured: object, expected: tuple[str, ...]) -> None:
    shaping: Final = route_host.shaping("anthropic/not-a-real-model", None, {"additional_drop_params": configured})

    assert shaping["additional_drop_params"] == expected


def test_native_request_rejections_map_to_the_public_400() -> None:
    from types import MappingProxyType

    from litellm.rust_bridge.messages.entrypoints import LiteLLMMessagesRequest

    request: Final = LiteLLMMessagesRequest(
        model="anthropic/claude-sonnet-5",
        messages=(),
        max_tokens=8,
        stream=None,
        api_key=None,
        api_base=None,
        custom_llm_provider=None,
        kwargs=MappingProxyType({}),
    )
    rejected: Final = ValueError("claude-sonnet-5 does not support top_k=5")
    rejected.messages_request_error = True  # pyright: ignore[reportAttributeAccessIssue]  # marker the native host sets

    mapped: Final = route_host.map_failure(rejected, request, "anthropic")

    assert isinstance(mapped, litellm.BadRequestError)
    assert mapped.status_code == 400
    assert "does not support top_k=5" in mapped.message
    assert mapped.model == "claude-sonnet-5"
    assert not isinstance(route_host.map_failure(ValueError("plain"), request, "anthropic"), litellm.BadRequestError)


def test_stream_hidden_params_projects_upstream_headers_the_way_the_python_handler_does() -> None:
    hidden: Final = route_host.stream_hidden_params(
        (("request-id", "req_upstream_123"), ("x-ratelimit-remaining-requests", "41"))
    )

    additional: Final = hidden["additional_headers"]
    assert isinstance(additional, dict)
    assert additional["llm_provider-request-id"] == "req_upstream_123"
    assert additional["x-ratelimit-remaining-requests"] == "41"
    assert additional["request-id"] == additional["llm_provider-request-id"]
