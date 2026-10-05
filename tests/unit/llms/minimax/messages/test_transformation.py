"""
Test MiniMax Anthropic-compatible API support
"""

from typing import Final

import pytest

import litellm
from litellm.llms.minimax.messages.transformation import MinimaxMessagesConfig
from litellm.types.router import GenericLiteLLMParams


def test_minimax_anthropic_config():
    """Test that MinimaxMessagesConfig is properly configured"""
    config = MinimaxMessagesConfig()

    # Test custom_llm_provider
    assert config.custom_llm_provider == "minimax"

    # Test get_api_base default
    api_base = config.get_api_base()
    assert api_base == "https://api.minimax.io/anthropic/v1/messages"

    # Test get_api_base with custom value
    custom_base = config.get_api_base(
        api_base="https://api.minimaxi.com/anthropic/v1/messages"
    )
    assert custom_base == "https://api.minimaxi.com/anthropic/v1/messages"


def test_minimax_provider_routing():
    """Test that minimax provider is properly routed"""
    from litellm.litellm_core_utils.get_llm_provider_logic import get_llm_provider

    # Test with minimax/ prefix
    model, provider, api_key, api_base = get_llm_provider(
        model="minimax/MiniMax-M2.1",
        api_base="https://api.minimax.io/anthropic/v1/messages",
    )
    assert provider == "minimax"
    assert model == "MiniMax-M2.1"


def test_minimax_provider_config_manager():
    """Test that ProviderConfigManager returns MinimaxMessagesConfig"""
    from litellm.types.utils import LlmProviders
    from litellm.utils import ProviderConfigManager

    config = ProviderConfigManager.get_provider_anthropic_messages_config(
        model="MiniMax-M2.1", provider=LlmProviders.MINIMAX
    )

    assert config is not None
    assert isinstance(config, MinimaxMessagesConfig)
    assert config.custom_llm_provider == "minimax"


if __name__ == "__main__":
    # Run basic tests that don't require API key
    print("Testing MiniMax Anthropic Config...")
    test_minimax_anthropic_config()
    print("✓ Config test passed")

    print("\nTesting MiniMax Provider Routing...")
    test_minimax_provider_routing()
    print("✓ Routing test passed")

    print("\nTesting MiniMax Provider Config Manager...")
    test_minimax_provider_config_manager()
    print("✓ Provider config manager test passed")

    print("\n✅ All basic tests passed!")


def test_minimax_messages_env_key_attached(monkeypatch):
    """Regression: an env-only MINIMAX_API_KEY must be attached on /v1/messages validation"""
    monkeypatch.setenv("MINIMAX_API_KEY", "test-minimax-env-key")
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    monkeypatch.delenv("ANTHROPIC_AUTH_TOKEN", raising=False)
    config = MinimaxMessagesConfig()
    headers, _ = config.validate_anthropic_messages_environment(
        headers={},
        model="MiniMax-M2.1",
        messages=[{"role": "user", "content": "hi"}],
        optional_params={},
        litellm_params={},
    )
    assert headers["x-api-key"] == "test-minimax-env-key"


def test_minimax_messages_explicit_key_wins_over_env(monkeypatch):
    monkeypatch.setenv("MINIMAX_API_KEY", "env-key")
    config = MinimaxMessagesConfig()
    headers, _ = config.validate_anthropic_messages_environment(
        headers={},
        model="MiniMax-M2.1",
        messages=[{"role": "user", "content": "hi"}],
        optional_params={},
        litellm_params={},
        api_key="param-key",
    )
    assert headers["x-api-key"] == "param-key"


@pytest.mark.parametrize("suffix", ("", "/", "/v1", "/v1/", "/v1/messages", "/v1/messages/"))
def test_minimax_messages_url_normalizes_versioned_base(suffix: str) -> None:
    base: Final = "https://minimax.example/anthropic"
    url: Final = MinimaxMessagesConfig().get_complete_url(
        api_base=f"{base}{suffix}",
        api_key=None,
        model="MiniMax-M3.1-Flash-Preview",
        optional_params={},
        litellm_params={},
    )
    assert url == f"{base}/v1/messages"


@pytest.mark.parametrize("thinking_type", ("adaptive", "disabled"))
def test_minimax_preserves_native_thinking_and_effort(thinking_type: str) -> None:
    thinking: Final = {"type": thinking_type}
    output_config: Final = {"effort": "high"}
    payload: Final = MinimaxMessagesConfig().transform_anthropic_messages_request(
        model="MiniMax-M3.1-Flash-Preview",
        messages=[{"role": "user", "content": "hello"}],
        anthropic_messages_optional_request_params={
            "max_tokens": 4096,
            "thinking": thinking,
            "output_config": output_config,
        },
        litellm_params=GenericLiteLLMParams(),
        headers={},
    )
    assert payload["thinking"] == thinking
    assert payload["output_config"] == output_config
