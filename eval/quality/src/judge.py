from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


@dataclass
class JudgeScore:
    dimension: str
    score: int
    justification: str


class Judge(Protocol):
    def evaluate(self, entry_content: str, rubric_prompt: str) -> list[JudgeScore]:
        """Score a knowledge entry against a rubric prompt."""
        ...
