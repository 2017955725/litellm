from typing import Final

import pytest

import litellm
from litellm.proxy.spend_tracking.baseline_accounting import (
    BaselineHistory,
    BaselineObservation,
    advance_baseline_history,
)
from litellm.proxy.spend_tracking.cache_history import estimate_cache_plan, normalize_cache_usage, prepare_cache_request
from litellm.proxy.spend_tracking.savings import BaselineCostSnapshot, price_baseline_comparison
from litellm.types.utils import ModelInfo, Usage

_PRICES: Final[ModelInfo] = {
    **litellm.get_model_info("gpt-6-astra", "openai"),
    "input_cost_per_token": 0.01,
    "output_cost_per_token": 0.03,
    "cache_read_input_token_cost": 0.001,
    "cache_creation_input_token_cost": 0.017,
}
_PROMPT: Final = "A long stable prefix with some reusable content. " * 1000


def _request(**overrides: object) -> dict[str, object]:
    return {"messages": [{"role": "user", "content": _PROMPT}], **overrides}


def _usage(*, read: int = 0, write: int = 8000) -> Usage:
    return normalize_cache_usage(
        Usage(
            prompt_tokens=8000,
            completion_tokens=20,
            total_tokens=8020,
            prompt_tokens_details={"cached_tokens": read, "cache_creation_tokens": write},
        )
    )


def _observation(
    request: dict[str, object],
    started: float = 10000.0,
    provider: str = "openai",
) -> BaselineObservation:
    prepared: Final = prepare_cache_request(request)
    assert prepared is not None
    usage: Final = _usage()
    captured: Final = estimate_cache_plan(
        prepared, "gpt-6-astra", provider, _PRICES, usage, lambda model, text: len(text)
    )
    assert captured is not None
    return BaselineObservation(
        request_id=str(started),
        started_at=started,
        available_at=started + 1,
        outcome="complete",
        baseline_equivalent=False,
        usage=usage,
        plan=captured.plan,
        cache_policy="estimated",
        cache_write_pricing="standard",
        assumptions=captured.assumptions,
    )


@pytest.mark.parametrize(
    "provider", ("openai", "azure", "gemini", "vertex_ai", "deepseek", "mistral", "custom_provider")
)
def test_switched_model_pays_actual_cold_write_while_baseline_reads_its_own_history(provider: str) -> None:
    first: Final = _observation(_request(), provider=provider)
    history, initial = advance_baseline_history(BaselineHistory(), (first,))
    _, second = advance_baseline_history(
        history, (first.model_copy(update={"request_id": "switch", "started_at": 10002.0, "available_at": 10003.0}),)
    )
    assert initial[0].usage is not None and second[0].usage is not None
    assert initial[0].usage.prompt_tokens_details.cache_creation_tokens == 8000
    assert getattr(initial[0].usage.prompt_tokens_details, "cache_creation_token_details", None) is None
    assert second[0].usage.prompt_tokens_details.cached_tokens == 8000
    assert second[0].usage.prompt_tokens_details.cache_creation_tokens == 0
    assert first.usage == _usage()
    actual: Final = 8000 * _PRICES["cache_creation_input_token_cost"] + 20 * _PRICES["output_cost_per_token"]
    snapshot: Final = BaselineCostSnapshot(
        model="gpt-6-astra",
        provider=provider,
        prices=_PRICES,
        actual_spend=actual,
        actual_token_cost=actual,
        classifier_cost=0.1,
    )
    priced: Final = price_baseline_comparison(snapshot, second[0].usage, second[0].provenance)
    assert priced is not None
    assert priced.actual == pytest.approx(actual + 0.1)
    assert priced.baseline == pytest.approx(
        8000 * _PRICES["cache_read_input_token_cost"] + 20 * _PRICES["output_cost_per_token"]
    )
    assert priced.savings < 0


@pytest.mark.parametrize("seconds,expected_reads", ((1799, 8000), (1800, 0)))
def test_unspecified_provider_lifetime_is_labeled_and_expires(seconds: int, expected_reads: int) -> None:
    first: Final = _observation(_request(), provider="custom_provider")
    assert "cache_lifetime_assumed_30m" in first.assumptions
    history, _ = advance_baseline_history(BaselineHistory(), (first,))
    _, estimates = advance_baseline_history(
        history,
        (
            first.model_copy(
                update={
                    "request_id": "later",
                    "started_at": first.started_at + seconds,
                    "available_at": first.available_at + seconds,
                }
            ),
        ),
    )
    assert estimates[0].usage is not None
    assert estimates[0].usage.prompt_tokens_details.cached_tokens == expected_reads
    assert estimates[0].usage.prompt_tokens_details.cache_creation_tokens == 8000 - expected_reads


@pytest.mark.parametrize("surface,kind", (("messages", "text"), ("input", "input_text")))
@pytest.mark.parametrize("ttl,seconds", (("5m", 300), ("1h", 3600)))
def test_message_marker_covers_the_last_content_block(surface: str, kind: str, ttl: str, seconds: int) -> None:
    request: Final = {
        surface: [
            {
                "role": "user",
                "cache_control": {"type": "ephemeral", "ttl": ttl},
                "content": [{"type": kind, "text": _PROMPT}, {"type": kind, "text": "suffix"}],
            }
        ],
        "prompt_cache_options": {"mode": "explicit"},
    }
    observation: Final = _observation(request)
    assert observation.plan is not None
    assert len(observation.plan.breakpoints) == 1
    marker: Final = observation.plan.breakpoints[0]
    assert (marker.ttl_seconds, marker.prefix_tokens) == (seconds, observation.usage.prompt_tokens)


@pytest.mark.parametrize("provider,control", (("anthropic", "5m"), ("bedrock", "1h")))
def test_duration_pricing_and_top_level_cache_control_use_configured_rates(provider: str, control: str) -> None:
    prices: Final[ModelInfo] = {**_PRICES, "cache_creation_input_token_cost_above_1hr": 0.024}
    raw: Final = prepare_cache_request(_request(cache_control={"type": "ephemeral", "ttl": control}))
    assert raw is not None
    estimated: Final = estimate_cache_plan(
        raw, "custom-assistant", provider, prices, _usage(), lambda model, text: len(text)
    )
    assert estimated is not None
    observation: Final = _observation(_request()).model_copy(
        update={"plan": estimated.plan, "cache_write_pricing": "duration"}
    )
    _, estimates = advance_baseline_history(BaselineHistory(), (observation,))
    usage: Final = estimates[0].usage
    assert usage is not None
    assert estimated.plan.breakpoints[0].ttl_seconds == (300 if control == "5m" else 3600)
    snapshot: Final = BaselineCostSnapshot(
        model="gpt-6-astra", provider="openai", prices=prices, actual_spend=1.0, actual_token_cost=1.0
    )
    priced: Final = price_baseline_comparison(snapshot, usage, estimates[0].provenance)
    assert priced is not None
    rate: Final = (
        prices["cache_creation_input_token_cost"]
        if control == "5m"
        else prices["cache_creation_input_token_cost_above_1hr"]
    )
    assert priced.baseline == pytest.approx(8000 * rate + 20 * prices["output_cost_per_token"])


def test_json_schema_fields_are_part_of_identity_and_oversized_requests_are_rejected() -> None:
    first: Final = _observation(
        _request(response_format={"json_schema": {"properties": {"cache_control": {"const": "first"}}}})
    )
    changed: Final = _observation(
        _request(response_format={"json_schema": {"properties": {"cache_control": {"const": "changed"}}}})
    )
    assert first.plan is not None and changed.plan is not None
    assert first.plan.breakpoints != changed.plan.breakpoints
    assert prepare_cache_request(_request(messages=[{"role": "user", "content": "x" * 5_000_000}])) is None


@pytest.mark.parametrize(
    "key,block", (("system", {"type": "text", "text": _PROMPT}), ("tools", {"name": "lookup", "description": _PROMPT}))
)
def test_explicit_system_and_tool_cache_writes_are_reused(key: str, block: dict[str, str]) -> None:
    first: Final = _observation(
        _request(**{key: [{**block, "cache_control": {"type": "ephemeral", "ttl": "1h"}}]}), provider="anthropic"
    )
    history, initial = advance_baseline_history(BaselineHistory(), (first,))
    _, replay = advance_baseline_history(
        history, (first.model_copy(update={"request_id": "replay", "started_at": 10002.0, "available_at": 10003.0}),)
    )
    assert initial[0].usage is not None and replay[0].usage is not None
    writes: Final = initial[0].usage.prompt_tokens_details.cache_creation_tokens
    assert 0 < writes < first.usage.prompt_tokens
    assert replay[0].usage.prompt_tokens_details.cached_tokens == writes
    assert replay[0].usage.prompt_tokens_details.cache_creation_tokens == 0
