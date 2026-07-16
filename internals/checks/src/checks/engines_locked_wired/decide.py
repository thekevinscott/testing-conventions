"""The engines-locked-wired decision — repo-only (#437).

The repo's workflows layer the third-party CI engine toolchain (`coverage`, `pytest`, `cosmic-ray`,
`maturin`) onto uv tool environments from the hash-pinned `.github/uv/engines.txt`
(`--with-requirements`), so every run resolves the same versions as a function of the commit under
test. A bare, floating `uv run --with pytest …` reintroduces the mutable external reference the lock
removed — the resolution drifts to whatever the index serves that day — so the decision holds one
fact: no workflow layers a CI engine with a bare `--with`.

Comment lines are dropped before the scan — a `--with coverage --with pytest` written in prose to
describe the run is context, not an invocation, and pinning the real `run:` line leaves that prose
in place.
"""
from __future__ import annotations

import re

from checks.config import CI_ENGINES

# `--with <engine>` with a trailing space is the bare, floating form; `--with-requirements <lock>`
# (a hyphen, no space) is the pinned form and is deliberately not matched.
_FLOATING = re.compile(r"--with (" + "|".join(re.escape(e) for e in CI_ENGINES) + r")\b")


def _uncommented(text: str) -> str:
    """`text` with its YAML comment lines dropped — a line whose first non-blank character is `#`."""
    return "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("#"))


def floating_engines(text: str) -> list[str]:
    """Every CI engine layered with a bare, floating `--with` in `text` (comments excluded)."""
    return _FLOATING.findall(_uncommented(text))


def decide(text: str) -> bool:
    """True when the workflow layers no CI engine with a bare, floating `--with`."""
    return not floating_engines(text)
