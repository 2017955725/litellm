use litellm_http::request::{with_header, without_headers};
use litellm_llms_types::providers::anthropic::{BetaProvider, BetaSet};

use crate::{anthropic::common_utils::existing_betas, base_llm::auth::Headers};

const BETA_HEADER: &str = "anthropic-beta";

/// What happens to the caller's and LiteLLM's `anthropic-beta` values before they leave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BetaPolicy {
    /// The host decides for itself which betas it accepts.
    Forward,
    /// Python's `update_headers_with_filtered_beta`: only the betas the provider accepts are
    /// sent, under the names it expects.
    Filter(BetaProvider),
    /// The host accepts no beta, as Python's filter treats a provider with no column.
    Drop,
}

impl BetaPolicy {
    /// Every casing of `anthropic-beta` is replaced by one sorted header, or removed when no
    /// beta survives.
    pub fn apply(self, headers: Headers) -> Headers {
        let provider = match self {
            Self::Forward => return headers,
            Self::Drop => return without_headers(headers, &[BETA_HEADER]),
            Self::Filter(provider) => provider,
        };
        let accepted: BetaSet = existing_betas(&headers)
            .iter()
            .filter_map(|beta| beta.on(provider))
            .collect();
        let headers = without_headers(headers, &[BETA_HEADER]);
        if accepted.is_empty() {
            return headers;
        }
        with_header(headers, BETA_HEADER, accepted.to_string())
    }
}

#[cfg(test)]
mod tests {
    use litellm_llms_types::providers::anthropic::AnthropicBeta;
    use rstest::rstest;

    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> Headers {
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
    }

    fn beta_header(beta: &AnthropicBeta) -> Headers {
        headers(&[("x-api-key", "k"), ("anthropic-beta", beta.as_str())])
    }

    fn beta_headers_after(policy: BetaPolicy, input: &[(&str, &str)]) -> Headers {
        policy.apply(headers(input))
    }

    #[test]
    fn filter_sends_each_beta_under_the_name_the_provider_expects() {
        for provider in BetaProvider::ALL {
            for beta in AnthropicBeta::KNOWN {
                let expected = match beta.on(*provider) {
                    Some(mapped) => beta_header(&mapped),
                    None => headers(&[("x-api-key", "k")]),
                };
                assert_eq!(
                    BetaPolicy::Filter(*provider).apply(beta_header(&beta)),
                    expected,
                    "{beta} on {provider}"
                );
            }
        }
    }

    #[test]
    fn filter_merges_every_casing_into_one_sorted_header() {
        let kept = AnthropicBeta::KNOWN
            .into_iter()
            .filter(|beta| beta.on(BetaProvider::Anthropic).as_ref() == Some(beta))
            .take(2)
            .collect::<Vec<_>>();
        let [first, second] = kept.as_slice() else {
            panic!("the anthropic column accepts at least two betas unchanged")
        };
        assert_eq!(
            beta_headers_after(
                BetaPolicy::Filter(BetaProvider::Anthropic),
                &[
                    ("ANTHROPIC-BETA", second.as_str()),
                    (
                        "anthropic-beta",
                        &format!("example-beta-2099-01-01,{first}")
                    ),
                ]
            ),
            headers(&[(
                "anthropic-beta",
                &BetaSet::from_iter([first.clone(), second.clone()]).to_string()
            )])
        );
    }

    #[rstest]
    #[case::forward(
        BetaPolicy::Forward,
        &[("Anthropic-Beta", "example-beta-2099-01-01"), ("x-api-key", "k")]
    )]
    #[case::drop(BetaPolicy::Drop, &[("x-api-key", "k")])]
    fn forward_and_drop_ignore_the_config(
        #[case] policy: BetaPolicy,
        #[case] expected: &[(&str, &str)],
    ) {
        assert_eq!(
            beta_headers_after(
                policy,
                &[
                    ("Anthropic-Beta", "example-beta-2099-01-01"),
                    ("x-api-key", "k")
                ]
            ),
            headers(expected)
        );
    }

    #[test]
    fn headers_without_a_beta_are_untouched() {
        for policy in BetaProvider::ALL
            .iter()
            .copied()
            .map(BetaPolicy::Filter)
            .chain([BetaPolicy::Forward, BetaPolicy::Drop])
        {
            assert_eq!(
                beta_headers_after(policy, &[("x-api-key", "k")]),
                headers(&[("x-api-key", "k")])
            );
        }
    }
}
