### The reusable workflow provisions the Python the package declares

**Summary**

The reusable workflow pinned `python-version: '3.12'` on every `astral-sh/setup-uv@v7` step, so a
package whose `requires-python` declared a higher floor (e.g. `>=3.14`) could not run a single
Python-environment gate — every provisioning job died at "Provision the Python suite environment
(uv)". The pin is dropped: `setup-uv` with no `python-version` leaves `UV_PYTHON` unset, and
`uv sync` resolves an interpreter satisfying the manifest's `requires-python`, downloading one if
the runner has none. The version is already declared in the package; deriving it matches how the
workflow derives the package manager, the pnpm version, and the native toolchain.

**Required changes**

_None._ The change is inside the reusable workflow; a `uses:` call needs no edit. The workflow
exposes no `python_version` input and never did.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A package whose `requires-python` excludes 3.12 (e.g. `>=3.14`) now provisions a satisfying
interpreter where it previously failed to provision at all. A package whose `requires-python`
permits 3.12 is unaffected — `uv` still resolves an interpreter satisfying the manifest, and 3.12
remains one it may pick. The runner downloads an interpreter only when no installed one satisfies
the manifest.

**Verification**

Run a workflow job that provisions Python (e.g. `unit coverage` on a Python package). The job log
shows `uv` resolving an interpreter satisfying the package's `requires-python` and completing the
"Provision the Python suite environment (uv)" step. A package with `requires-python = ">=3.14"`
provisions a 3.14 interpreter where it previously errored at the same step.
