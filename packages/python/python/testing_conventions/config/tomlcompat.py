"""The version-conditional TOML loader: stdlib ``tomllib`` on 3.11+, the ``tomli`` backport before."""
try:
    from tomllib import load
except ImportError:
    from tomli import load
