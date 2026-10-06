from dataclasses import replace
from typing import Final

import pytest

import litellm
from litellm.llms.prompt_cache_estimation import estimate_cache_plan, normalize_cache_usage, prepare_cache_request
from litellm.proxy.spend_tracking.baseline_accounting import (
    BaselineHistory,
    BaselineObservation,
    advance_baseline_history,
)
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


@pytest.mark.parametrize("modality", ("audio_tokens", "image_tokens", "video_tokens"))
def test_multimodal_estimates_keep_history_and_price_cold_warm_expired_tokens(modality: str) -> None:
    usage: Final = normalize_cache_usage(
        Usage(
            prompt_tokens=8000,
            completion_tokens=20,
            total_tokens=8020,
            prompt_tokens_details={modality: 2000, "cached_tokens": 0, "cache_creation_tokens": 0},
        )
    )
    first: Final = _observation(_request(), provider="gemini").model_copy(update={"usage": usage})
    history, cold = advance_baseline_history(BaselineHistory(), (first,))
    warmed, warm = advance_baseline_history(
        history, (first.model_copy(update={"request_id": "warm", "started_at": 10002.0, "available_at": 10003.0}),)
    )
    _, expired = advance_baseline_history(
        warmed, (first.model_copy(update={"request_id": "expired", "started_at": 12000.0, "available_at": 12001.0}),)
    )
    prices: Final[ModelInfo] = {
        **_PRICES,
        "cache_read_input_audio_token_cost": 0.002,
        "input_cost_per_audio_token": 0.02,
        "input_cost_per_image_token": 0.03,
        "input_cost_per_video_token": 0.04,
    }
    snapshot: Final = BaselineCostSnapshot(
        model="test-model", provider="gemini", prices=prices, actual_spend=100.0, actual_token_cost=100.0
    )
    for estimate, cost in (
        (cold[0], 8000 * 0.017 + 20 * 0.03),
        (warm[0], 6000 * 0.001 + 2000 * (0.002 if modality == "audio_tokens" else 0.001) + 20 * 0.03),
        (expired[0], 8000 * 0.017 + 20 * 0.03),
    ):
        assert estimate.usage is not None, estimate.reason
        assert (priced := price_baseline_comparison(snapshot, estimate.usage, estimate.provenance)) is not None
        assert priced.baseline == pytest.approx(cost)


@pytest.mark.parametrize(
    "reason", ("estimation_capacity_exhausted", "baseline_estimation_timeout", "unsupported_cache_request")
)
def test_skipped_estimation_preserves_paid_cache_without_refreshing_it(reason: str) -> None:
    first: Final = _observation(_request())
    history, _ = advance_baseline_history(BaselineHistory(), (first,))
    skipped: Final = first.model_copy(
        update={"request_id": "skipped", "started_at": 10002.0, "available_at": 10003.0, "plan": None, "reason": reason}
    )
    retained, missing = advance_baseline_history(history, (skipped,))
    assert missing[0].usage is None
    assert retained.entries == history.entries
    _, followup = advance_baseline_history(
        retained, (first.model_copy(update={"request_id": "followup", "started_at": 10004.0, "available_at": 10005.0}),)
    )
    assert followup[0].usage is not None and followup[0].usage.prompt_tokens_details.cached_tokens == 8000
    _, expired = advance_baseline_history(
        retained, (first.model_copy(update={"request_id": "expired", "started_at": 11800.0, "available_at": 11801.0}),)
    )
    assert expired[0].usage is not None and expired[0].usage.prompt_tokens_details.cached_tokens == 0


def test_partial_multimodal_cache_replaces_observed_splits_without_double_charging() -> None:
    usage: Final = normalize_cache_usage(
        Usage(
            prompt_tokens=8000,
            completion_tokens=20,
            total_tokens=8020,
            prompt_tokens_details={
                "audio_tokens": 2000,
                "image_tokens": 1000,
                "video_tokens": 500,
                "cached_tokens": 2000,
                "cached_tokens_details": {
                    "audio_tokens": 800,
                    "image_tokens": 300,
                    "text_tokens": 900,
                },
            },
        )
    )
    original: Final = _observation(_request(), provider="gemini")
    assert original.plan is not None
    plan: Final = replace(original.plan, breakpoints=(replace(original.plan.breakpoints[0], prefix_tokens=4000),))
    first: Final = original.model_copy(update={"usage": usage, "plan": plan})
    history, cold = advance_baseline_history(BaselineHistory(), (first,))
    _, warm = advance_baseline_history(
        history,
        (
            first.model_copy(
                update={
                    "request_id": "warm",
                    "started_at": 10002.0,
                    "available_at": 10003.0,
                }
            ),
        ),
    )
    prices: Final[ModelInfo] = {
        **_PRICES,
        "cache_read_input_audio_token_cost": 0.002,
        "input_cost_per_audio_token": 0.02,
        "input_cost_per_image_token": 0.03,
        "input_cost_per_video_token": 0.04,
    }
    snapshot: Final = BaselineCostSnapshot(
        model="test-model",
        provider="gemini",
        prices=prices,
        actual_spend=100.0,
        actual_token_cost=100.0,
    )
    ordinary: Final = 2250 * 0.01 + 1000 * 0.02 + 500 * 0.03 + 250 * 0.04 + 20 * 0.03
    for value, expected in (
        (usage, 3600 * 0.01 + 1200 * 0.02 + 700 * 0.03 + 500 * 0.04 + 1200 * 0.001 + 800 * 0.002 + 20 * 0.03),
        (cold[0].usage, ordinary + 4000 * 0.017),
        (warm[0].usage, ordinary + 3000 * 0.001 + 1000 * 0.002),
    ):
        assert value is not None
        assert (priced := price_baseline_comparison(snapshot, value, "modeled")) is not None
        assert priced.baseline == pytest.approx(expected)
    assert usage.prompt_tokens_details.cached_tokens_details.audio_tokens == 800
