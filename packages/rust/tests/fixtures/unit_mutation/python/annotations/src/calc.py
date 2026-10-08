from __future__ import annotations

# The future import holds every annotation unevaluated, the state 3.14 reaches on its own
# (PEP 649). Without it the mutants die at `def` time on the 3.12 CI runs this fixture must
# reproduce on, and only a 3.14 run would show the survivors.

TOTALS: dict[str, int | None] = {"total": 11 - 4}


def add(a: int | None, b: int = 10 - 4) -> int | None:
    if a is None:
        return None
    return a + b
