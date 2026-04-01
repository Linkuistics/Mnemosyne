from __future__ import annotations

import yaml

from anthropic import Anthropic

from eval.quality.src.judge import Judge, JudgeScore


class ClaudeJudge:
    """Judge implementation using the Anthropic SDK."""

    def __init__(self, model: str = "claude-haiku-4-5-20251001"):
        self.client = Anthropic()
        self.model = model

    def evaluate(self, entry_content: str, rubric_prompt: str) -> list[JudgeScore]:
        message = self.client.messages.create(
            model=self.model,
            max_tokens=1024,
            messages=[
                {
                    "role": "user",
                    "content": (
                        f"{rubric_prompt}\n\n"
                        f"---\n\n"
                        f"Entry to evaluate:\n\n"
                        f"{entry_content}\n\n"
                        f"---\n\n"
                        f"Respond with ONLY a YAML list. Each item must have: "
                        f"dimension, score (integer 1-5), justification (one sentence)."
                    ),
                }
            ],
        )

        response_text = message.content[0].text
        return self._parse_response(response_text)

    def _parse_response(self, text: str) -> list[JudgeScore]:
        # Strip markdown code fences if present
        text = text.strip()
        if text.startswith("```"):
            lines = text.split("\n")
            text = "\n".join(lines[1:])
            if text.endswith("```"):
                text = text[:-3]

        parsed = yaml.safe_load(text)
        if not isinstance(parsed, list):
            return []

        scores = []
        for item in parsed:
            if isinstance(item, dict) and all(
                k in item for k in ("dimension", "score", "justification")
            ):
                scores.append(
                    JudgeScore(
                        dimension=str(item["dimension"]),
                        score=int(item["score"]),
                        justification=str(item["justification"]),
                    )
                )
        return scores
