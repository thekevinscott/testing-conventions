# Getting Started

`testing-conventions` enforces a library's testing standards as deterministic, bright-line
checks — the kind an agent (or a hurried human) can't quietly cross while keeping CI green.
The fastest way to adopt it is the reusable GitHub Actions workflow: drop in one job and
**every check runs with sane defaults — no config file required.** Reach for a config file
later, and only to refine.

## The fastest start: one workflow, no config

Add a workflow to your repo that calls the reusable one, pinned to a tag:

```yaml
# .github/workflows/conventions.yml
name: Conventions
on: [pull_request]

jobs:
  conventions:
    uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0
    with:
      languages: '["python", "typescript"]'   # the languages your library ships
      path: src                                # the directory to scan
```

That's the whole install. On every pull request it runs the published binary and **opts your
library into every check we offer**, each as its own matrix job that fails the build — with the
offending files in the log — on any violation:

- **Colocated test** — every source file has a colocated, matching-named unit test (every language).
- **Unit coverage** — the unit suite meets a coverage floor (Python and TypeScript).
- **Integration lint** — integration tests use the right mocking mechanism & style (Python).

No `testing-conventions.toml` is needed. The coverage floor falls back to the language's **sane
default** — Python `branch = true, fail_under = 85`; TypeScript `lines = 80, branches = 75,
functions = 80, statements = 80` — so a new library is enforced from day one. (`languages` and
`path` are the only inputs without a universal default: the checks differ by language, and every
project lays out its sources differently.)

## Customize with a config file (optional)

When a default isn't right — you want a stricter floor, or a file genuinely shouldn't be tested —
add a `testing-conventions.toml` at your repo root. One file drives every rule, and the workflow
picks it up automatically:

```toml
# Tighten the Python floor past the default 85:
[python]
coverage = { branch = true, fail_under = 95 }

# Exempt a launcher shim — explicit, and a reason is required:
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test", "coverage"]
reason = "thin launcher; logic in run(), tested in run_test.py"

[typescript]
coverage = { lines = 90, branches = 85, functions = 90, statements = 90 }
```

Anything you omit keeps its default, so a config can be as small as a single tightened floor or
one exemption. See [Configuration](./reference/#configuration) for every key and
[Exempt a file](./guide/exemptions) for the exemption rules.

## Prefer to run it yourself? Install the CLI

The workflow just runs a single binary, published to three registries under the same name.
Install it whichever way matches your toolchain:

```sh
# Rust (crates.io)
cargo install testing-conventions

# Python (PyPI) — the wheel bundles the binary
pip install testing-conventions

# Node (npm) — as a dev dependency, run via npx
npm install --save-dev testing-conventions
```

Confirm it's available (prefix `npx` if you installed it as an npm dev dependency):

```sh
testing-conventions --version
```

Then call any rule directly, naming the language with the required `--language` flag. For example
the **colocated test** rule — every source file must have a colocated unit test named after it:

```sh
# Python: foo.py must have a sibling foo_test.py
testing-conventions unit colocated-test --language python src/

# TypeScript: foo-bar.ts must have a sibling foo-bar.test.ts
testing-conventions unit colocated-test --language typescript src/
```

When every source file is paired, the command prints nothing and exits `0`. When a file is
missing its twin, each orphan is listed on stderr and the command exits `1`:

```
missing colocated unit test: src/widget.ts
missing colocated unit test: src/pkg/orphan.ts
error: 2 source file(s) missing a colocated unit test
```

That non-zero exit is the whole point: in CI an orphaned (or missing) test can't slip through
green. `unit coverage` runs the same way, and its `--config` is optional too — omit it and the
default floor above applies.

## Next steps

- [Guides](./guide/) — task-oriented recipes (enforce a rule, wire it into CI, exempt a file).
- [Reference](./reference/) — every subcommand, flag, exit code, and config key.
