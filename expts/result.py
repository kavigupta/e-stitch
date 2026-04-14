"""Common result format shared by all compression methods (ours/babble/stitch).

Each method wrapper returns a :class:`Result`, which captures the corpus
sizes before/after compression, the learned library, and a wall-clock time
measured uniformly (so cross-method comparison is apples-to-apples).
"""

from dataclasses import dataclass, asdict, field
from typing import Any


@dataclass
class Result:
    """Uniform result record for a single compression run."""

    method: str
    """One of ``"enum"``, ``"smc"``, ``"babble"``, ``"stitch"``."""

    domain: str
    """The cogsci domain name (e.g. ``"dials"``)."""

    initial_cost: int
    """Corpus AST size before any compression is applied."""

    final_cost: int
    """Corpus AST size after the learned library is applied."""

    compression_ratio: float
    """``initial_cost / final_cost`` computed uniformly from the two costs."""

    elapsed_secs: float
    """Wall-clock time for the run (subprocess duration)."""

    library: list[str]
    """Human-readable strings for each learned abstraction."""

    extra: dict[str, Any] = field(default_factory=dict)
    """Method-specific fields that don't fit the common schema."""

    def to_dict(self) -> dict:
        """Return the plain-dict representation for JSON serialization."""
        return asdict(self)

    def summary_line(self) -> str:
        """Return a single-line summary suitable for printing."""
        return (
            f"{self.method}/{self.domain}: "
            f"{self.initial_cost} -> {self.final_cost} "
            f"(ratio {self.compression_ratio:.2f}, time {self.elapsed_secs:.1f}s, "
            f"{len(self.library)} lib)"
        )


def ratio(initial: int, final: int) -> float:
    """Safe division for a compression ratio; returns ``inf`` when ``final == 0``."""
    return float("inf") if final == 0 else initial / final
