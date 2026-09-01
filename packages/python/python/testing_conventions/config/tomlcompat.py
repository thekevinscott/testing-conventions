"""The version-conditional TOML loader.

``tomllib`` is stdlib from 3.11 and ``tomli`` backports it before, so one import branch is dead on
any single interpreter — but both are reachable from one test run: ``tomlcompat_test.py`` forces
the fallback by making ``import tomllib`` raise and re-importing the module. Kept to two bare
imports with no logic, so nothing testable hides behind it.
"""
try:
    from tomllib import load
except ImportError:
    from tomli import load
