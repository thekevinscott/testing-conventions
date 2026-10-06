from __future__ import annotations

# The future import holds the annotations unevaluated on every interpreter, the state 3.14
# reaches on its own (PEP 649); dropping it makes this fixture reproduce on 3.14 alone.

TOTALS: dict[str, int | None] = {"total": 11 - 4}


def add(a: int | None, b: int = 10 - 4) -> int | None:
    if a is None:
        return None
    return a + b
