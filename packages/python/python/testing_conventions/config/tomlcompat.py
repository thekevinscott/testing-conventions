"""The version-conditional TOML loader.

One of these import branches is dead on any single interpreter (``tomllib`` is
stdlib from 3.11; ``tomli`` backports it before), so the module can't be fully
exercised on one test run — that's why it carries a narrow exemption. It is kept
to this irreducible minimum (one import, no logic) so nothing testable hides
behind the exemption.
"""
try:
    import tomllib as _toml
except ImportError:  # pragma: no cover - 3.9 / 3.10 fall back to tomli (shipped by coverage)
    import tomli as _toml

load = _toml.load
