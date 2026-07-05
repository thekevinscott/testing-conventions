#!/usr/bin/env python3
"""Assert the detect action routes Python into the unit-lint matrix (#189; epic #302, #305).

Repo-only: this module exists *only in this repository*; it is not part of the published
`testing-conventions` tool. It backs the `detect-routes-python` job of the self-test workflow
(.github/workflows/testing-conventions-selftest.yml). That job runs the repo-only `detect`
composite action over the `clean` fixture and hands this script the action's
`isolation_languages` output — a compact JSON array such as `["python"]` or `["python","rust"]`
— which this script asserts routes Python into the unit-lint matrix.

The value arrives as `argv[1]` (never an environment side-channel: the workflow templates
`${{ steps.detect.outputs.isolation_languages }}` into the argument). Living as a standalone,
colocated-tested script — rather than an inline `run: |` block — lets the routing assertion carry
real unit + e2e tests, and keeps its logic off the untested, `${{ }}`-templated `run:` path.
"""
import json
import sys
from typing import Optional


def routes_python(isolation_languages: str) -> Optional[str]:
    """Error message when Python is not routed into the unit-lint matrix, else None.

    `isolation_languages` is the compact JSON array the detect action emits (e.g. `["python"]`,
    `["python","rust"]`, `[]`). Python routes in when it is a member of that array.
    """
    try:
        languages = json.loads(isolation_languages)
    except (json.JSONDecodeError, ValueError):
        languages = None
    if isinstance(languages, list) and "python" in languages:
        return None
    return (
        "the detect action did not route Python into isolation_languages "
        f"(got: {isolation_languages})"
    )


def main(argv: list) -> int:
    value = argv[1]
    problem = routes_python(value)
    if problem is not None:
        print(f"::error::{problem}")
        return 1
    print(f"isolation_languages={value}")
    print("Python routed into the unit-lint matrix")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
