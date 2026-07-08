"""Shared paths and expected values the checks target (#321) — one source of truth, so a literal
isn't copied into every check module.
"""
REUSABLE_WORKFLOW = ".github/workflows/testing-conventions.yml"
DOGFOOD_HELPERS_WORKFLOW = ".github/workflows/dogfood-github-helpers.yml"

# The outputs `./.github/actions/detect` must produce for the monorepo TS fixture (#277).
TS_FIXTURE_PACKAGE_ROOT = ".github/selftest/monorepo/packages/ts"
TS_FIXTURE_PACKAGE_MANAGER = "npm"
TS_FIXTURE_PROVISION_RUST = "false"
TS_FIXTURE_CONFIG = ".github/selftest/monorepo/packages/ts/testing-conventions.toml"

# The outputs `./.github/actions/detect` must produce for the monorepo Python fixture (#277).
PY_FIXTURE_PACKAGE_ROOT = ".github/selftest/monorepo/packages/py"
PY_FIXTURE_PYTHON_ENV = "uv"
PY_FIXTURE_CONFIG = ".github/selftest/monorepo/packages/py/testing-conventions.toml"

# The repo-only caller workflows that build the hermetic-cli artifact (#356): every `uses:` call
# of the reusable workflow in these files must `needs: [build-cli]` so the artifact exists before
# the called workflow's rule jobs download it.
SELFTEST_WORKFLOW = ".github/workflows/testing-conventions-selftest.yml"
DOGFOOD_WORKFLOW = ".github/workflows/dogfood.yml"
