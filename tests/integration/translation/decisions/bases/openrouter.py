from typing import Final

from integration.translation.case import TranslationTestCase

TYPESAFE_JEV_1_13_TEST_CASE: Final = TranslationTestCase(
    scenario="basic",
    litellm_endpoint="/v1/decisions",
    litellm_request={
        "model": "openrouter/typesafe/jev-1.13",
        "state": {"ticket": "The export job hangs at 99% and never finishes", "component": "billing"},
        "questions": {
            "defect": {"type": "noul", "instructions": "Is this a defect?"},
            "severity": {
                "type": "choice",
                "instructions": "How severe is it?",
                "criteria": {"low": "cosmetic", "high": "blocks users"},
            },
            "confidence": {"type": "score", "instructions": "How sure are you?", "criteria": ["unsure", "sure"]},
        },
        "cache": {"no-cache": True},
    },
    expected_provider_endpoint="/alpha/decisions",
    expected_provider_headers={
        "authorization": "Bearer synthetic-openrouter-key",
        "content-type": "application/json",
    },
    expected_provider_request={
        "model": "typesafe/jev-1.13",
        "state": {"ticket": "The export job hangs at 99% and never finishes", "component": "billing"},
        "questions": {
            "defect": {"type": "noul", "instructions": "Is this a defect?"},
            "severity": {
                "type": "choice",
                "instructions": "How severe is it?",
                "criteria": {"low": "cosmetic", "high": "blocks users"},
            },
            "confidence": {"type": "score", "instructions": "How sure are you?", "criteria": ["unsure", "sure"]},
        },
    },
    mock_provider_response={
        "model": "typesafe/jev-1.13-20260917",
        "answers": {
            "defect": {"type": "noul", "noul": 0.81},
            "severity": {
                "type": "choice",
                "choice": "high",
                "probabilities": {"low": 0.01, "high": 0.99},
                "confidence": 0.99,
            },
            "confidence": {
                "type": "score",
                "score": 0.5,
                "legend": {"0": "unsure", "1": "sure"},
                "probabilities": {"0": 0.5, "1": 0.5},
                "confidence": 0,
            },
        },
        "usage": {"input_tokens": 377, "output_tokens": 62, "cost": 1.5834e-05},
        "id": "gen-dec-1791323839-GA15kY0nt34oiJ7srfki",
        "provider": "TypeSafe",
    },
    expected_litellm_response={
        "model": "typesafe/jev-1.13-20260917",
        "answers": {
            "defect": {"type": "noul", "noul": 0.81},
            "severity": {
                "type": "choice",
                "choice": "high",
                "probabilities": {"low": 0.01, "high": 0.99},
                "confidence": 0.99,
            },
            "confidence": {
                "type": "score",
                "score": 0.5,
                "legend": {"0": "unsure", "1": "sure"},
                "probabilities": {"0": 0.5, "1": 0.5},
                "confidence": 0,
            },
        },
        "usage": {"input_tokens": 377, "output_tokens": 62, "cost": 1.5834e-05},
        "id": "gen-dec-1791323839-GA15kY0nt34oiJ7srfki",
        "provider": "TypeSafe",
    },
)
