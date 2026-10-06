from typing import Final

from integration.translation.case import TranslationTestCase

JEV_1_13_0_TEST_CASE: Final = TranslationTestCase(
    scenario="basic",
    litellm_endpoint="/v1/decisions",
    litellm_request={
        "model": "typesafe/jev-1.13.0",
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
    expected_provider_endpoint="/v1/systemone",
    expected_provider_headers={
        "authorization": "Bearer synthetic-typesafe-key",
        "content-type": "application/json",
    },
    expected_provider_request={
        "model": "jev-1.13.0",
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
        "model": "jev-1.13.0",
        "answers": {
            "defect": {"type": "noul", "noul": 0.78},
            "severity": {
                "type": "choice",
                "choice": "high",
                "confidence": 0.99,
                "probabilities": {"high": 0.99, "low": 0.01},
            },
            "confidence": {
                "type": "score",
                "score": 0.51,
                "confidence": 0.03,
                "legend": {"0": "unsure", "1": "sure"},
                "probabilities": {"0": 0.49, "1": 0.51},
            },
        },
        "usage": {"input_tokens": 377, "output_tokens": 62},
    },
    expected_litellm_response={
        "model": "jev-1.13.0",
        "answers": {
            "defect": {"type": "noul", "noul": 0.78},
            "severity": {
                "type": "choice",
                "choice": "high",
                "confidence": 0.99,
                "probabilities": {"high": 0.99, "low": 0.01},
            },
            "confidence": {
                "type": "score",
                "score": 0.51,
                "confidence": 0.03,
                "legend": {"0": "unsure", "1": "sure"},
                "probabilities": {"0": 0.49, "1": 0.51},
            },
        },
        "usage": {"input_tokens": 377, "output_tokens": 62},
    },
)
