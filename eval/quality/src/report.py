from __future__ import annotations

import json
import statistics
from dataclasses import dataclass, field

from eval.quality.src.judge import JudgeScore
from eval.quality.src.structural import StructuralResult


@dataclass
class EntryReport:
    filename: str
    scores: list[JudgeScore]
    structural: StructuralResult | None = None


@dataclass
class AggregateReport:
    entries: list[EntryReport]
    structural_results: list[StructuralResult] = field(default_factory=list)

    def dimension_stats(self) -> dict[str, dict[str, float]]:
        """Compute mean, median, std for each dimension."""
        by_dim: dict[str, list[int]] = {}
        for entry in self.entries:
            for score in entry.scores:
                by_dim.setdefault(score.dimension, []).append(score.score)

        stats = {}
        for dim, scores in sorted(by_dim.items()):
            stats[dim] = {
                "mean": round(statistics.mean(scores), 1),
                "median": statistics.median(scores),
                "std": round(statistics.stdev(scores), 1) if len(scores) > 1 else 0.0,
            }
        return stats

    def format_human(self, verbose: bool = False) -> str:
        lines = []
        dim_stats = self.dimension_stats()

        lines.append(f"Entry Quality (N={len(self.entries)} entries):")
        for dim, stats in dim_stats.items():
            lines.append(
                f"  {dim + ':':20s} mean={stats['mean']:.1f}  "
                f"median={stats['median']:.0f}  std={stats['std']:.1f}"
            )

        # Structural completeness
        valid = sum(1 for r in self.structural_results if r.valid)
        total = len(self.structural_results)
        issues = [r for r in self.structural_results if not r.valid]
        lines.append(f"\n  Structural completeness: {valid}/{total} valid")
        if issues:
            lines.append(f"  ({len(issues)} issues)")
            for r in issues:
                for err in r.errors:
                    lines.append(f"    - {r.filename}: {err}")

        if verbose:
            # Lowest scoring entries
            entry_avgs = []
            for entry in self.entries:
                if entry.scores:
                    avg = statistics.mean(s.score for s in entry.scores)
                    entry_avgs.append((entry.filename, avg, entry.scores))
            entry_avgs.sort(key=lambda x: x[1])

            lines.append("\n  Lowest scoring entries:")
            for filename, avg, scores in entry_avgs[:5]:
                score_str = ", ".join(
                    f"{s.dimension}={s.score}" for s in scores
                )
                lines.append(f"    - {filename}: {score_str}")

        return "\n".join(lines)

    def format_json(self) -> str:
        return json.dumps(
            {
                "entry_count": len(self.entries),
                "dimension_stats": self.dimension_stats(),
                "structural": {
                    "valid": sum(1 for r in self.structural_results if r.valid),
                    "total": len(self.structural_results),
                },
                "entries": [
                    {
                        "filename": e.filename,
                        "scores": [
                            {
                                "dimension": s.dimension,
                                "score": s.score,
                                "justification": s.justification,
                            }
                            for s in e.scores
                        ],
                    }
                    for e in self.entries
                ],
            },
            indent=2,
        )
