from __future__ import annotations

import argparse
import os
import random
import statistics
import sys

from eval.quality.src.config import get_model, get_provider
from eval.quality.src.judge import JudgeScore
from eval.quality.src.report import AggregateReport, EntryReport
from eval.quality.src.rubric import format_rubric_prompt, load_rubric
from eval.quality.src.structural import check_directory, check_entry


def create_judge(provider: str, model: str):
    if provider == "claude":
        from eval.quality.src.providers.claude import ClaudeJudge

        return ClaudeJudge(model=model)
    else:
        print(f"Unknown provider: {provider}", file=sys.stderr)
        sys.exit(1)


def evaluate_entries(
    entries_dir: str,
    rubric_path: str,
    provider: str,
    model: str,
    single_pass: bool,
    verbose: bool,
) -> AggregateReport:
    rubric = load_rubric(rubric_path)
    judge = create_judge(provider, model)

    # Structural checks first (no API calls)
    structural_results = check_directory(entries_dir)

    entry_reports = []
    for filename in sorted(os.listdir(entries_dir)):
        if not filename.endswith(".md"):
            continue
        filepath = os.path.join(entries_dir, filename)
        with open(filepath) as f:
            content = f.read()

        if verbose:
            print(f"  Evaluating {filename}...", file=sys.stderr)

        # Pass 1: standard dimension order
        prompt1 = format_rubric_prompt(rubric, shuffle=False)
        scores1 = judge.evaluate(content, prompt1)

        if single_pass:
            entry_reports.append(EntryReport(filename=filename, scores=scores1))
            continue

        # Pass 2: shuffled dimension order (variance reduction)
        prompt2 = format_rubric_prompt(rubric, shuffle=True)
        scores2 = judge.evaluate(content, prompt2)

        # Average scores across passes, keep justification from pass 1
        averaged = _average_scores(scores1, scores2)
        entry_reports.append(EntryReport(filename=filename, scores=averaged))

    return AggregateReport(entries=entry_reports, structural_results=structural_results)


def _average_scores(
    pass1: list[JudgeScore], pass2: list[JudgeScore]
) -> list[JudgeScore]:
    """Average scores from two passes, keeping pass 1 justifications."""
    scores2_map = {s.dimension: s.score for s in pass2}
    averaged = []
    for s1 in pass1:
        s2_score = scores2_map.get(s1.dimension, s1.score)
        avg = round(statistics.mean([s1.score, s2_score]))
        averaged.append(
            JudgeScore(
                dimension=s1.dimension,
                score=avg,
                justification=s1.justification,
            )
        )
    return averaged


def main():
    parser = argparse.ArgumentParser(description="Mnemosyne knowledge quality evaluator")
    parser.add_argument(
        "--corpus",
        default=os.path.join(os.path.dirname(__file__), "..", "..", "corpus", "entries"),
        help="Path to entries directory (default: eval/corpus/entries)",
    )
    parser.add_argument("--store", help="Evaluate live knowledge store instead of corpus")
    parser.add_argument(
        "--rubric",
        default=os.path.join(
            os.path.dirname(__file__), "..", "rubrics", "entry_quality.yaml"
        ),
        help="Path to rubric YAML",
    )
    parser.add_argument("--provider", help="LLM provider (default: claude)")
    parser.add_argument("--model", help="Model ID")
    parser.add_argument(
        "--single-pass", action="store_true", help="Skip variance reduction"
    )
    parser.add_argument("--json", action="store_true", help="Output in JSON format")
    parser.add_argument("--verbose", action="store_true", help="Per-entry breakdown")

    args = parser.parse_args()

    entries_dir = args.store if args.store else args.corpus
    provider = get_provider(args.provider)
    model = get_model(args.model)

    report = evaluate_entries(
        entries_dir=entries_dir,
        rubric_path=args.rubric,
        provider=provider,
        model=model,
        single_pass=args.single_pass,
        verbose=args.verbose,
    )

    if args.json:
        print(report.format_json())
    else:
        print(report.format_human(verbose=args.verbose))


if __name__ == "__main__":
    main()
