"""The version-conditional TOML loader.

One of these import branches is dead on any single interpreter (``tomllib`` is
stdlib from 3.11; ``tomli`` backports it before), so the module can't be fully
exercised on one test run — that's why it carries a narrow exemption. It is kept
to two bare imports with no logic, so nothing testable hides behind it.
"""
try:
    from tomllib import load
except ImportError:  # pragma: no cover - 3.9 / 3.10 fall back to tomli (shipped by coverage)
    from tomli import load
