#!/usr/bin/env python3
"""Assert `dogfood-github-helpers.yml` invokes every Python arm over the .github helpers (#237).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `github-helpers-wired` self-test job in
`.github/workflows/testing-conventions-selftest.yml`, which guards that the dogfood gate for the
repo's own `.github/` helper scripts still runs all five Python arms — the four whole-tree arms
(colocated-test, unit lint, unit coverage, integration lint) plus the diff-scoped mutation rung
(`unit mutation --base`). A dropped arm would silently stop dogfooding `detect.py` /
`move_major_tag.py`.

It lives as a standalone, colocated-tested script — rather than an inline `run: |` block — because
inline workflow bash is untested prose and is vulnerable to the GitHub-Actions `${{ }}` templating
trap (a `run:` string is templated before the shell sees it). The pure `find_missing_arm` holds the
assertion logic and is unit-tested in isolation; `main` reads the workflow file and reports.
"""
import re
import sys
from pathlib import Path
from typing import Optional

# Default target: the dogfood workflow that holds the .github helpers to the package conventions.
DEFAULT_PATH = ".github/workflows/dogfood-github-helpers.yml"

# The five Python arms the dogfood gate must invoke. Each entry is (human-readable arm name,
# regex asserting the arm is wired). The mutation arm is diff-scoped, so its pattern additionally
# requires the `--base` flag — a bare `unit mutation` without `--base` is the whole-tree form the
# gate deliberately does not use.
ARMS: list[tuple[str, str]] = [
    ("unit colocated-test", r"unit colocated-test"),
    ("unit lint", r"unit lint"),
    ("unit coverage", r"unit coverage"),
    ("integration lint", r"integration lint"),
    ("unit mutation --base", r"unit mutation .*--base"),
]


def find_missing_arm(text: str) -> Optional[str]:
    """Return the name of the first Python arm the workflow text does not wire, else None.

    Pure: takes the workflow file's contents as a string and returns the missing arm's name (or
    None when all five are present). This is the whole assertion, exercised by the unit suite.
    """
    for name, pattern in ARMS:
        if re.search(pattern, text) is None:
            return name
    return None


def main(argv: list[str]) -> int:
    path = argv[1] if len(argv) > 1 else DEFAULT_PATH
    text = Path(path).read_text()
    missing = find_missing_arm(text)
    if missing is not None:
        print(
            f"::error::{path} does not run the '{missing}' Python arm over the .github helpers — "
            "the dogfood gate (#237) must invoke unit colocated-test, unit lint, unit coverage, "
            "integration lint, and the diff-scoped unit mutation --base"
        )
        return 1
    print(f"{path} runs all five Python arms")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
