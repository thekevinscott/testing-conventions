"""Shared paths and expected values the checks target, so no literal is copied into a check module."""
REUSABLE_WORKFLOW = ".github/workflows/testing-conventions.yml"
NODE_PACKAGE_MANIFEST = "packages/node/package.json"

# The outputs `./.github/actions/detect` must produce for the monorepo TS fixture.
TS_FIXTURE_PACKAGE_ROOT = ".github/selftest/monorepo/packages/ts"
TS_FIXTURE_PACKAGE_MANAGER = "npm"
TS_FIXTURE_PROVISION_RUST = "false"
TS_FIXTURE_CONFIG = ".github/selftest/monorepo/packages/ts/testing-conventions.toml"

# The outputs `./.github/actions/detect` must produce for the monorepo Python fixture.
PY_FIXTURE_PACKAGE_ROOT = ".github/selftest/monorepo/packages/py"
PY_FIXTURE_PYTHON_ENV = "uv"
PY_FIXTURE_CONFIG = ".github/selftest/monorepo/packages/py/testing-conventions.toml"

# The repo-only caller workflows that build the hermetic-cli artifact.
SELFTEST_WORKFLOW = ".github/workflows/testing-conventions-selftest.yml"
DOGFOOD_WORKFLOW = ".github/workflows/dogfood.yml"

# The hermetic binary each red-path job downloads, spliced ahead of a check's subcommand argv.
HERMETIC_CLI = ["./hermetic-cli/testing-conventions"]
