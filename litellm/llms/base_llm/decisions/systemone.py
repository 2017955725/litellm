"""The Jev / System One wire shape and its translation to and from the OpenAI Decisions shape.

System One (TypeSafe, Perplexity, OpenRouter, Cloudflare Clef, Strands Decider) takes
{"model", "state", "questions": {name: question}} and answers with {"model", "answers": {name: answer}, "usage"}.
Predicates are `noul` questions, choice options are a `criteria` map, score levels are a `criteria` list.
"""

from collections.abc import Mapping, Sequence
from typing import Final, Literal, TypeAlias

from pydantic import ConfigDict, TypeAdapter

from litellm.llms.base_llm.chat.transformation import BaseLLMException
from litellm.types.decisions import (
    ChoiceAnswer,
    ChoiceProbability,
    ChoiceQuestion,
    DecisionAnswer,
    DecisionInputMessage,
    DecisionQuestion,
    DecisionsRequest,
    DecisionsResponse,
    DecisionsUsage,
    PredicateAnswer,
    PredicateQuestion,
    ScoreAnswer,
    ScoreProbability,
    ScoreQuestion,
)
from litellm.types.llms.base import LiteLLMPydanticObjectBase

SystemOneJSON: TypeAlias = str | Mapping[str, object] | Sequence[object]


class SystemOneNoulAnswer(LiteLLMPydanticObjectBase):
    type: Literal["noul"]
    noul: float

    model_config = ConfigDict(extra="allow", frozen=True)


class SystemOneChoiceAnswer(LiteLLMPydanticObjectBase):
    type: Literal["choice"]
    choice: str
    confidence: float
    probabilities: Mapping[str, float]

    model_config = ConfigDict(extra="allow", frozen=True)


class SystemOneScoreAnswer(LiteLLMPydanticObjectBase):
    type: Literal["score"]
    score: float
    confidence: float
    probabilities: Mapping[str, float]

    model_config = ConfigDict(extra="allow", frozen=True)


SystemOneAnswer: TypeAlias = SystemOneNoulAnswer | SystemOneChoiceAnswer | SystemOneScoreAnswer


class SystemOneUsage(LiteLLMPydanticObjectBase):
    input_tokens: int = 0
    output_tokens: int = 0

    model_config = ConfigDict(extra="allow", frozen=True)


class SystemOneResponse(LiteLLMPydanticObjectBase):
    model: str | None = None
    answers: Mapping[str, SystemOneAnswer]
    usage: SystemOneUsage | None = None

    model_config = ConfigDict(extra="allow", frozen=True)


SYSTEM_ONE_RESPONSE_ADAPTER: Final[TypeAdapter[SystemOneResponse]] = TypeAdapter(SystemOneResponse)


def _unsupported(what: str, custom_llm_provider: str) -> BaseLLMException:
    return BaseLLMException(
        status_code=400,
        message=f"Decisions provider '{custom_llm_provider}' does not support {what}",
    )


def question_keys(questions: Sequence[DecisionQuestion], custom_llm_provider: str) -> tuple[str, ...]:
    """System One keys questions and answers by name, so unnamed questions get a positional key."""
    keys: Final = tuple(
        question.name if question.name is not None else f"q{index}" for index, question in enumerate(questions)
    )
    if len(set(keys)) != len(keys):
        raise BaseLLMException(
            status_code=400,
            message=f"Decisions provider '{custom_llm_provider}' requires a unique name per question",
        )
    return keys


def system_one_state(request: DecisionsRequest, custom_llm_provider: str) -> str:
    if isinstance(request.input, str):
        return request.input
    return "\n".join(_message_text(message, custom_llm_provider) for message in request.input)


def _message_text(message: DecisionInputMessage, custom_llm_provider: str) -> str:
    if isinstance(message.content, str):
        return message.content
    texts: list[str] = []
    for part in message.content:
        if part.type != "input_text":
            raise _unsupported("input_image parts", custom_llm_provider)
        texts.append(part.text)
    return "\n".join(texts)


def system_one_question(question: DecisionQuestion, custom_llm_provider: str) -> dict[str, object]:
    if isinstance(question, PredicateQuestion):
        return {"type": "noul", "instructions": question.instructions}
    if isinstance(question, ChoiceQuestion):
        criteria: dict[str, str | None] = {}
        for choice in question.choices:
            if not isinstance(choice.value, str):
                raise _unsupported("boolean choice values", custom_llm_provider)
            criteria[choice.value] = choice.description
        return {"type": "choice", "instructions": question.instructions, "criteria": criteria}
    return {
        "type": "score",
        "instructions": question.instructions,
        "criteria": [level.description if level.description is not None else level.label for level in question.levels],
    }


def system_one_request(model: str, request: DecisionsRequest, custom_llm_provider: str) -> dict[str, object]:
    keys: Final = question_keys(request.questions, custom_llm_provider)
    return {
        "model": model,
        "state": system_one_state(request, custom_llm_provider),
        "questions": {
            key: system_one_question(question, custom_llm_provider)
            for key, question in zip(keys, request.questions, strict=True)
        },
    }


def _answer_for(
    key: str,
    question: DecisionQuestion,
    answers: Mapping[str, SystemOneAnswer],
    custom_llm_provider: str,
) -> DecisionAnswer:
    answer: Final = answers.get(key)
    if isinstance(question, PredicateQuestion) and isinstance(answer, SystemOneNoulAnswer):
        return PredicateAnswer(type="predicate", name=question.name, probability=answer.noul)
    if isinstance(question, ChoiceQuestion) and isinstance(answer, SystemOneChoiceAnswer):
        return ChoiceAnswer(
            type="choice",
            name=question.name,
            choice=answer.choice,
            probabilities=[
                ChoiceProbability(value=choice.value, probability=answer.probabilities.get(str(choice.value), 0.0))
                for choice in question.choices
            ],
            confidence=answer.confidence,
        )
    if isinstance(question, ScoreQuestion) and isinstance(answer, SystemOneScoreAnswer):
        return ScoreAnswer(
            type="score",
            name=question.name,
            score=answer.score,
            probabilities=[
                ScoreProbability(value=index, label=level.label, probability=answer.probabilities.get(str(index), 0.0))
                for index, level in enumerate(question.levels)
            ],
            confidence=answer.confidence,
        )
    raise BaseLLMException(
        status_code=500,
        message=f"Decisions provider '{custom_llm_provider}' returned no {question.type} answer for question '{key}'",
    )


def decisions_response(
    system_one: SystemOneResponse,
    request: DecisionsRequest,
    custom_llm_provider: str,
) -> DecisionsResponse:
    keys: Final = question_keys(request.questions, custom_llm_provider)
    usage: Final = system_one.usage if system_one.usage is not None else SystemOneUsage()
    return DecisionsResponse(
        model=system_one.model,
        answers=[
            _answer_for(key, question, system_one.answers, custom_llm_provider)
            for key, question in zip(keys, request.questions, strict=True)
        ],
        usage=DecisionsUsage(
            input_tokens=usage.input_tokens,
            output_tokens=usage.output_tokens,
            total_tokens=usage.input_tokens + usage.output_tokens,
        ),
    )
