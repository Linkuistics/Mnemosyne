from __future__ import annotations

import os


def get_provider(cli_provider: str | None) -> str:
    return cli_provider or os.environ.get("MNEMOSYNE_EVAL_PROVIDER", "claude")


def get_model(cli_model: str | None) -> str:
    return cli_model or os.environ.get(
        "MNEMOSYNE_EVAL_MODEL", "claude-haiku-4-5-20251001"
    )
