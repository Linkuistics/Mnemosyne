from __future__ import annotations

import random
from typing import Any

import yaml


def load_rubric(path: str) -> dict[str, Any]:
    """Load a rubric YAML file."""
    with open(path) as f:
        return yaml.safe_load(f)


def format_rubric_prompt(rubric: dict[str, Any], shuffle: bool = False) -> str:
    """Format a rubric into a prompt string for LLM evaluation.

    If shuffle=True, randomize dimension order for variance reduction.
    """
    dimensions = list(rubric.get("dimensions", {}).items())
    if shuffle:
        random.shuffle(dimensions)

    lines = [
        f"Evaluate this knowledge entry on the following dimensions.",
        f"For each dimension, provide a score (1-5) and a one-sentence justification.",
        f"Return your response as a YAML list with fields: dimension, score, justification.",
        "",
    ]

    for dim_name, dim_spec in dimensions:
        lines.append(f"## {dim_name}")
        lines.append(dim_spec.get("description", ""))
        lines.append("")
        for score, anchor in sorted(dim_spec.get("anchors", {}).items(), reverse=True):
            lines.append(f"  {score}: {anchor}")
        lines.append("")

    return "\n".join(lines)
