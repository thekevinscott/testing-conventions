### The devDependency copy no longer shadows the release the workflow runs

**Summary**

`npx <name>` runs a binary already present in the checkout's `node_modules/.bin` in preference to
fetching one. The suite-executing jobs (`unit-coverage`, `coverage-changed`, `mutation`,
`packaging`) run `pnpm install --frozen-lockfile` / `npm ci` before they invoke the CLI, so a repo
carrying `testing-conventions` as a devDependency executed that copy — a version it picked, frozen
wherever its lockfile put it. A caret does not widen below `0.1.0`, so `^0.0.91` means exactly
0.0.91 across every release. The jobs that install nothing (`static`, `e2e-verify`) fetched from
the registry, so two jobs in one run enforced two different rulesets.

Every invocation now names a prefix outside the checkout —
`npm --prefix "$RUNNER_TEMP" exec --yes -- "testing-conventions"` — which is the tree npm searches
before it reaches for the registry. The registry and the `version` input are the only inputs to
resolution.

**Required changes**

_None._ The change is inside the reusable workflow; a `uses:` call needs no edit.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A repo that carries `testing-conventions` in its own manifest moves from that pinned version to the
current release in the install-first jobs. The older the pin, the further the jump, and a repo that
passes today can go red on its next run. That red is accurate: the newer rule is the one the
workflow means to run.

The manifest entry keeps its other jobs. Local runs (`npx testing-conventions …` in the repo) still
resolve it, and so does anything else that depends on it. The workflow simply stops reading it. A
local run reaching the registry instead — no such entry in the repo — needs node 24 or newer, the
engine the package declares: npm resolves a bare name to the newest release the running node
satisfies.

To choose the version yourself, name it on the call:

```yaml
with:
  version: '0.0.91'
```

That pins every job to one release, so a staged walk forward is a series of edits to that one line.

**Verification**

Each job's log opens with the version that produced its result:

```
testing-conventions 0.0.105
```

Read that line in a job that installs first — `Unit-test coverage`, or `Unit mutation` — and compare
it against the `testing-conventions` entry in your own manifest. They are now free to differ; before
this change they matched.
