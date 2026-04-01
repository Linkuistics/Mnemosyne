from __future__ import annotations

import re
from dataclasses import dataclass, field

import yaml


VALID_CONFIDENCE = {"high", "medium", "low", "prospective"}
ISO_DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


@dataclass
class StructuralResult:
    filename: str
    valid: bool
    errors: list[str] = field(default_factory=list)


def check_entry(content: str, filename: str) -> StructuralResult:
    """Check structural completeness of a knowledge entry."""
    errors: list[str] = []

    # Split frontmatter
    content = content.strip()
    if not content.startswith("---"):
        errors.append("Missing YAML frontmatter delimiter")
        return StructuralResult(filename=filename, valid=False, errors=errors)

    parts = content.split("---", 2)
    if len(parts) < 3:
        errors.append("Missing closing frontmatter delimiter")
        return StructuralResult(filename=filename, valid=False, errors=errors)

    yaml_str = parts[1]
    body = parts[2].strip()

    try:
        fm = yaml.safe_load(yaml_str)
    except yaml.YAMLError as e:
        errors.append(f"Invalid YAML: {e}")
        return StructuralResult(filename=filename, valid=False, errors=errors)

    if not isinstance(fm, dict):
        errors.append("Frontmatter is not a mapping")
        return StructuralResult(filename=filename, valid=False, errors=errors)

    # Required fields
    if "title" not in fm:
        errors.append("Missing required field: title")

    if "tags" not in fm:
        errors.append("Missing required field: tags")
    elif not isinstance(fm["tags"], list) or len(fm["tags"]) == 0:
        errors.append("Tags must be a non-empty list")

    if "created" not in fm:
        errors.append("Missing required field: created")
    elif not ISO_DATE_RE.match(str(fm["created"])):
        errors.append(f"Invalid date format in created: {fm['created']}")

    if "confidence" not in fm:
        errors.append("Missing required field: confidence")
    elif str(fm["confidence"]).lower() not in VALID_CONFIDENCE:
        errors.append(
            f"Invalid confidence value: {fm['confidence']} "
            f"(must be one of {sorted(VALID_CONFIDENCE)})"
        )

    # Body check
    if not body:
        errors.append("Body is empty")

    # Origins check for high/medium confidence
    confidence = str(fm.get("confidence", "")).lower()
    if confidence in ("high", "medium"):
        origins = fm.get("origins", [])
        if not origins:
            errors.append(
                f"Origins should be present for {confidence} confidence entries"
            )

    return StructuralResult(
        filename=filename,
        valid=len(errors) == 0,
        errors=errors,
    )


def check_directory(entries_dir: str) -> list[StructuralResult]:
    """Check all .md files in a directory."""
    import os

    results = []
    for filename in sorted(os.listdir(entries_dir)):
        if not filename.endswith(".md"):
            continue
        filepath = os.path.join(entries_dir, filename)
        with open(filepath) as f:
            content = f.read()
        results.append(check_entry(content, filename))
    return results
