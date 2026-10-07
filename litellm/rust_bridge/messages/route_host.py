from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import asdict, dataclass
from typing import Final, Literal, cast  # noqa: TID251  # narrows the normalized native payload to the public TypedDict

import httpx
from pydantic import TypeAdapter, ValidationError

import litellm
from litellm.litellm_core_utils.core_helpers import normalize_drop_params
from litellm.llms.anthropic.pass_through.utils import is_reasoning_auto_summary_enabled
from litellm.rust_bridge import failures
from litellm.rust_bridge.messages.entrypoints import LiteLLMMessagesRequest
from litellm.types.llms.anthropic_messages.anthropic_response import AnthropicMessagesResponse

_DROP_PATHS: Final = TypeAdapter(list[object])
_METADATA_SOURCE: Final = TypeAdapter(dict[object, object])


@dataclass(frozen=True, slots=True)
class BedrockMetadataSource:
    identity: tuple[tuple[str, str], ...]
    spend_logs: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class BedrockRequestMetadataInput:
    allowed_fields: tuple[str, ...]
    sources: tuple[BedrockMetadataSource, ...]


@dataclass(frozen=True, slots=True)
class BedrockMessagesConnection:
    api_base: str | None
    region: str | None
    model_id: str | None
    workspace_id: str | None


@dataclass(frozen=True, slots=True)
class EffortTiers:
    minimal: bool
    low: bool
    medium: bool
    high: bool
    xhigh: bool
    max: bool


@dataclass(frozen=True, slots=True)
class ModelCapabilities:
    supports_reasoning: bool
    supports_adaptive_thinking: bool
    thinking_always_on: bool
    supports_legacy_thinking: bool
    supports_output_config: bool
    supports_sampling_params: bool
    supports_speed: bool
    effort_tiers: EffortTiers
    supports_mid_conversation_system: bool = False
    supports_cache_control_ttl: bool = False
    supports_native_structured_output: bool = False
    supports_tool_search: bool = False
    effort_ceiling: Literal["low", "medium", "high", "xhigh", "max"] | None = None


@dataclass(frozen=True, slots=True)
class MessagesShaping:
    capabilities: ModelCapabilities
    drop_params: bool
    reasoning_auto_summary: bool
    additional_drop_params: Sequence[str]
    bedrock_request_metadata: BedrockRequestMetadataInput | None = None
    bedrock_connection: BedrockMessagesConnection | None = None


def response(value: Mapping[str, object]) -> AnthropicMessagesResponse:
    return cast(  # cast-ok: AnthropicMessagesResponse is a TypedDict over the normalized native payload
        AnthropicMessagesResponse,
        dict(value),
    )


def stream_hidden_params(headers: Sequence[tuple[str, str]]) -> Mapping[str, object]:
    from litellm.llms.anthropic.pass_through.messages.streaming_iterator import (
        anthropic_messages_stream_hidden_params,
    )

    return anthropic_messages_stream_hidden_params(httpx.Headers(list(headers)))


def arguments(request: LiteLLMMessagesRequest) -> Mapping[str, object]:
    return request.kwargs


def map_failure(error: Exception, request: LiteLLMMessagesRequest, request_provider: str) -> Exception:
    if getattr(error, "messages_request_error", False):
        return litellm.BadRequestError(
            message=str(error),
            model=request.model.removeprefix(f"{request_provider}/"),
            llm_provider=request_provider,
        )
    return failures.map_native_failure(error, request.model, request_provider, arguments(request), request.api_base)


def _resolved_provider(model: str, custom_llm_provider: str | None) -> tuple[str, str]:
    try:
        resolved_model, provider, _, _ = litellm.get_llm_provider(model=model, custom_llm_provider=custom_llm_provider)
    except Exception:  # noqa: BLE001  # an unroutable model still shapes as a bare Anthropic id
        return model, custom_llm_provider or "anthropic"
    return resolved_model, provider


def model_capabilities(model: str, custom_llm_provider: str | None) -> ModelCapabilities:
    from litellm.llms.anthropic.chat.transformation import AnthropicConfig
    from litellm.llms.anthropic.common_utils import AnthropicModelInfo
    from litellm.llms.bedrock.common_utils import (
        _bedrock_model_supports,
        _get_bedrock_output_config_effort_ceiling,
        bedrock_supports_tool_search,
        is_claude_4_5_on_bedrock,
    )

    resolved_model, provider = _resolved_provider(model, custom_llm_provider)

    def supports(flag: str) -> bool:
        return AnthropicModelInfo._supports_model_capability(model, flag, provider)  # pyright: ignore[reportPrivateUsage]  # same probes the Python transform runs; forking them would drift

    def tier(level: str) -> bool:
        return AnthropicConfig._supports_effort_level(model, level, provider)  # pyright: ignore[reportPrivateUsage]  # same probe the Python transform runs

    return ModelCapabilities(
        supports_reasoning=supports("supports_reasoning"),
        supports_adaptive_thinking=supports("supports_adaptive_thinking"),
        thinking_always_on=supports("thinking_always_on"),
        supports_legacy_thinking=supports("supports_legacy_thinking"),
        supports_output_config=supports("supports_output_config"),
        supports_sampling_params=AnthropicModelInfo._supports_sampling_params(resolved_model),  # pyright: ignore[reportPrivateUsage]  # same gate the handler applies
        supports_speed=AnthropicConfig._model_supports_speed_param(resolved_model, provider),  # pyright: ignore[reportPrivateUsage]  # same gate the handler applies
        supports_mid_conversation_system=supports("supports_mid_conversation_system"),
        supports_cache_control_ttl=provider == "bedrock" and is_claude_4_5_on_bedrock(resolved_model),
        supports_native_structured_output=provider == "bedrock"
        and _bedrock_model_supports(resolved_model, "supports_native_structured_output"),
        supports_tool_search=provider == "bedrock" and bedrock_supports_tool_search(resolved_model),
        effort_ceiling=_get_bedrock_output_config_effort_ceiling(resolved_model) if provider == "bedrock" else None,
        effort_tiers=EffortTiers(
            minimal=tier("minimal"),
            low=tier("low"),
            medium=tier("medium"),
            high=tier("high"),
            xhigh=tier("xhigh"),
            max=tier("max"),
        ),
    )


def _drop_params(kwargs: Mapping[str, object]) -> bool:
    return bool(litellm.drop_params) or normalize_drop_params(kwargs.get("drop_params")) is True


def _additional_drop_params(kwargs: Mapping[str, object]) -> tuple[str, ...]:
    try:
        configured: Final = _DROP_PATHS.validate_python(kwargs.get("additional_drop_params"))
    except ValidationError:
        return ()
    return tuple(path for path in configured if isinstance(path, str))


def _text_metadata_pairs(value: object) -> tuple[tuple[str, str], ...]:
    try:
        source: Final = _METADATA_SOURCE.validate_python(value)
    except ValidationError:
        return ()
    return tuple((key, text) for key, text in source.items() if isinstance(key, str) and isinstance(text, str))


def _bedrock_metadata_source(value: object) -> BedrockMetadataSource:
    try:
        source: Final = _METADATA_SOURCE.validate_python(value)
    except ValidationError:
        return BedrockMetadataSource((), ())
    return BedrockMetadataSource(
        identity=tuple((key, text) for key, text in source.items() if isinstance(key, str) and isinstance(text, str)),
        spend_logs=_text_metadata_pairs(source.get("spend_logs_metadata")),
    )


def _bedrock_request_metadata(
    model: str, custom_llm_provider: str | None, kwargs: Mapping[str, object]
) -> BedrockRequestMetadataInput | None:
    configured: Final = litellm.bedrock_request_metadata_fields
    if not configured:
        return None
    _, provider = _resolved_provider(model, custom_llm_provider)
    if provider != "bedrock":
        return None
    return BedrockRequestMetadataInput(
        allowed_fields=tuple(configured),
        sources=tuple(_bedrock_metadata_source(kwargs.get(name)) for name in ("metadata", "litellm_metadata")),
    )


def _bedrock_connection(
    model: str, custom_llm_provider: str | None, kwargs: Mapping[str, object]
) -> BedrockMessagesConnection | None:
    _, provider = _resolved_provider(model, custom_llm_provider)
    if provider != "bedrock":
        return None

    def text(name: str) -> str | None:
        value: Final = kwargs.get(name)
        return value if isinstance(value, str) else None

    return BedrockMessagesConnection(
        api_base=text("aws_bedrock_runtime_endpoint"),
        region=text("aws_region_name"),
        model_id=text("model_id"),
        workspace_id=text("aws_bedrock_project_id"),
    )


def shaping(model: str, custom_llm_provider: str | None, kwargs: Mapping[str, object]) -> dict[str, object]:
    return asdict(
        MessagesShaping(
            capabilities=model_capabilities(model, custom_llm_provider),
            drop_params=_drop_params(kwargs),
            reasoning_auto_summary=is_reasoning_auto_summary_enabled(),
            additional_drop_params=_additional_drop_params(kwargs),
            bedrock_request_metadata=_bedrock_request_metadata(model, custom_llm_provider, kwargs),
            bedrock_connection=_bedrock_connection(model, custom_llm_provider, kwargs),
        )
    )
