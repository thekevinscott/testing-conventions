"""The engines-drift decision — repo-only (#439).

Pinning the CI engine toolchain (#437) removes the one thing the old floating `--with` bought: CI
exercising the engine a consumer would resolve today. This restores that early-warning signal
outside the required-check set — a scheduled job recompiles `.github/uv/engines.in` at latest and
this decision compares it to the committed lock. Any version difference is drift: the ecosystem
moved past the pin, and it names what changed so the pin can be advanced deliberately (not tangled
into an unrelated bump).

Pure text in, drift lines out. The recompile-at-latest is a plain tool invocation the workflow
runs; the parse-and-compare — the part that decides red — lives here, tested.
"""
from __future__ import annotations

import re

# A top-level pin line: `name==version` anchored at the line start, so the indented `--hash=` lines
# and `# comment` header a compiled lock carries are never mistaken for a pin. The version stops at
# whitespace or a `;` environment marker.
_PIN = re.compile(r"^([A-Za-z0-9][A-Za-z0-9._-]*)==([^\s;]+)", re.M)


def parse_pins(text: str) -> dict[str, str]:
    """Map every `name==version` pin in a compiled requirements text to its version, names
    normalized (lowercased, `_`/`.`→`-`) so `cosmic_ray` and `cosmic-ray` compare equal."""
    return {re.sub(r"[._]", "-", name).lower(): version for name, version in _PIN.findall(text)}


def drift(committed_text: str, latest_text: str) -> list[str]:
    """Every package whose pinned version differs between the committed lock and a latest
    resolution, as `name: <committed> → <latest>` lines (a missing side shown as `—`)."""
    committed, latest = parse_pins(committed_text), parse_pins(latest_text)
    lines = []
    for name in sorted(set(committed) | set(latest)):
        was, now = committed.get(name), latest.get(name)
        if was != now:
            lines.append(f"{name}: {was or '—'} → {now or '—'}")
    return lines
