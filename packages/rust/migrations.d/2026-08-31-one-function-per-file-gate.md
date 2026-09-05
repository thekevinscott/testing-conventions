### `unit one-function-per-file` runs in the reusable workflow

**Summary**

The `unit one-function-per-file` rule shipped as a CLI subcommand, but nothing in
`.github/workflows/testing-conventions.yml` invoked it, so adopting the workflow ran seven gates
and left the eighth on the shelf. It now runs as a step of the `Static checks (<language>)` job.
A Python or TypeScript tree that has never been held to the rule sees it for the first time on
the next run, at the shipped default of one line.

**Required changes**

A tree whose source does not pass at `max_lines = 1` declares the threshold that passes today in
its `testing-conventions.toml` and walks it down as files split. Run the rule locally to find the
lowest number that is green:

```toml
[python]
one_function_per_file = { max_lines = 12 }

[typescript]
one_function_per_file = { max_lines = 8 }
```

A file whose functions genuinely belong together takes a reason-required exemption instead:

```toml
[[python.exempt]]
path = "widget/registry.py"
rules = ["one-function-per-file"]
reason = "generated dispatch table: one small constructor per supported node type"
```

A call that names `gates` explicitly adds `one-function-per-file` to the list to run it:

```yaml
# before
gates: '["colocated-test", "unit-lint", "integration-lint"]'
# after
gates: '["colocated-test", "one-function-per-file", "unit-lint", "integration-lint"]'
```

**Deprecations removed**

_None._

**Behavior changes without code changes**

A `uses:` call that names no `gates` now runs one more gate. Python and TypeScript are checked at
`max_lines = 1`. Rust reports that the rule is not enabled and exits 0 until a
`[rust].one_function_per_file` table names a threshold.

**Verification**

Run the rule the way the workflow does, from the package root:

```sh
npx testing-conventions unit one-function-per-file --language python \
  --config testing-conventions.toml src
```

The CLI runs on node 24 or newer — npm resolves a bare name to the newest release the running node
satisfies.

A green tree exits 0 and prints only the version banner. A red one names each violation and the
function already holding the file:

```
src/widget.py:41: one-function-per-file — `render` runs 9 lines, and `parse` already holds this file; move it to its own module
error: 1 function(s) sharing a file with another function over the 1-line threshold (move each to its own module, or add an `exempt` entry with a reason)
```

In CI, the `Static checks (<language>)` job carries a `Check one function per file` step.
