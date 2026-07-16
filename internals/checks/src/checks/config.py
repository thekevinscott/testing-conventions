"""Shared paths and expected values the checks target (#321) — one source of truth, so a literal
isn't copied into every check module.
"""
REUSABLE_WORKFLOW = ".github/workflows/testing-conventions.yml"
DOGFOOD_HELPERS_WORKFLOW = ".github/workflows/dogfood-github-helpers.yml"

# The directory holding every workflow the repo runs — the scan surface for engines-locked-wired.
WORKFLOWS_DIR = ".github/workflows"

# The CI Python engine toolchain and the hash-pinned lock that pins it (#437): the third-party
# tools the repo's workflows layer onto uv tool environments (the test runner, the coverage and
# mutation engines, the maturin build backend). A workflow layers them by pointing uv at the lock
# (`--with-requirements .github/uv/engines.txt`); reintroducing a bare, floating `--with coverage`
# is the regression engines-locked-wired guards against.
CI_ENGINE_LOCK = ".github/uv/engines.txt"
CI_ENGINES = ("coverage", "pytest", "cosmic-ray", "maturin")

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

# The CLI-invocation prefix the direct-drive red-path checks run (#379): the hermetic binary the
# caller's `build-cli` job stages and each red-path job downloads to `./hermetic-cli/`, so the
# self-test validates this branch's CLI, not the published npm-latest one. Spliced ahead of each
# check's subcommand argv (`[*HERMETIC_CLI, "unit", "coverage", …]`).
HERMETIC_CLI = ["./hermetic-cli/testing-conventions"]
