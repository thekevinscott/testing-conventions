"""Where one authored list and one factual surface disagree — pure."""

from __future__ import annotations

from collections.abc import Iterable


def surface_drift(surface: str, claimed: Iterable[str], actual: Iterable[str]) -> list[str]:
    """Every disagreement between `claimed` and `actual`, naming the side each name is missing from.

    Both directions are reported because they are different bugs: a name the inventory claims and
    `surface` lacks is an entry outliving the thing it documents, and a name `surface` has and the
    inventory lacks is a check nobody wrote down. Reporting only the second would let a deleted
    check keep its page.
    """
    only_claimed = set(claimed) - set(actual)
    only_actual = set(actual) - set(claimed)
    return [
        *(f'{surface} has no "{name}", and the inventory claims it' for name in sorted(only_claimed)),
        *(f'"{name}" is in {surface} and not in the inventory' for name in sorted(only_actual)),
    ]
