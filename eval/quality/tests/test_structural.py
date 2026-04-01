import pytest
from eval.quality.src.structural import check_entry, StructuralResult


VALID_ENTRY = """\
---
title: Test Entry
tags: [rust, async]
created: 2025-06-01
last_validated: 2025-06-01
confidence: high
origins:
  - project: test-project
    date: 2025-06-01
    context: "Testing"
supersedes: []
---

This is the body content.
"""

MISSING_TITLE = """\
---
tags: [rust]
created: 2025-06-01
last_validated: 2025-06-01
confidence: high
supersedes: []
---

Body.
"""

INVALID_CONFIDENCE = """\
---
title: Bad Confidence
tags: [rust]
created: 2025-06-01
last_validated: 2025-06-01
confidence: extreme
supersedes: []
---

Body.
"""

HIGH_NO_ORIGINS = """\
---
title: High Without Origins
tags: [rust]
created: 2025-06-01
last_validated: 2025-06-01
confidence: high
supersedes: []
---

Body without origins.
"""

PROSPECTIVE_NO_ORIGINS = """\
---
title: Prospective Entry
tags: [rust]
created: 2025-06-01
last_validated: 2025-06-01
confidence: prospective
source: horizon-scan
supersedes: []
---

Body.
"""


def test_valid_entry():
    result = check_entry(VALID_ENTRY, "test.md")
    assert result.valid
    assert result.errors == []


def test_missing_title():
    result = check_entry(MISSING_TITLE, "test.md")
    assert not result.valid
    assert any("title" in e.lower() for e in result.errors)


def test_invalid_confidence():
    result = check_entry(INVALID_CONFIDENCE, "test.md")
    assert not result.valid
    assert any("confidence" in e.lower() for e in result.errors)


def test_high_confidence_without_origins():
    result = check_entry(HIGH_NO_ORIGINS, "test.md")
    assert not result.valid
    assert any("origins" in e.lower() for e in result.errors)


def test_prospective_without_origins_is_ok():
    result = check_entry(PROSPECTIVE_NO_ORIGINS, "test.md")
    assert result.valid
