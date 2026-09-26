---
description: "The workflow-lint check — a GitHub Actions `run:` or `github-script` body must be wiring, not a program. Covers the markers that make a body logic, the glue ceiling, which files are scanned, and why there is no exemption."
---

# `workflow-lint`

A `run:` block is the one place in a repository where code escapes every other gate. Nothing
imports it, so no test covers it; nothing builds it, so no compiler or linter reads it; you cannot
run it locally without a runner. This check keeps logic out of it.

## Why this check exists

Every other check on this list assumes code lives somewhere a test can reach. A shell script pasted
into YAML lives nowhere: it has no module, no caller, and no way to be exercised except by pushing
a commit and watching a job. So it is written once, half-verified by the one green run that
followed, and then it drifts — because the next person to touch it is also guessing.

The failure mode is specific. A loop over package names silently publishes nothing when the list is
empty. A `case` arm that never matches falls through to success. A `sed` expression that stops
matching leaves a version string unreplaced, and the release ships. None of these fail loudly, and
none of them are visible in review, because a reviewer reading YAML is reading configuration and
skims it as such.

Moving the logic into a tested package in the repository's own language costs one function and one
test. What you get back is a thing you can run on your laptop, step through, and cover — and a
`run:` line short enough that reviewing it is reading it.

## What it enforces {#enforces}

Every step's `run:` body, and the `script:` of an `actions/github-script` step, must read as
wiring. A body is flagged when it carries any of these:

| Marker | Why it is logic |
| --- | --- |
| `for`, `while`, `until`, `select` at a command position | iteration — the empty-list case is invisible |
| `case` at a command position | multi-branch dispatch — the unmatched arm is invisible |
| `awk` running anywhere in the body | a text transformation worth its own test |
| `sed` running anywhere in the body | as above |
| more than **12** command lines | past a dozen commands it is a script whatever its control flow |

A keyword counts only as the **first word** of a line, so `before_hook` and `casements=3` are
untouched. `awk` and `sed` count wherever a command can start — bare, or after a `|`, `;`, `&&`, or
`$(` — so `echo x | awk …` is caught, while `echo 'use awk for this'` is not.

The line count ignores bookkeeping: blank lines, `#` comments, and a `set -…` prologue do not
count against the ceiling.

### What stays inline

The check is built to tolerate the glue a consumer's step would carry anyway. None of this is
flagged:

- A toolchain install, a `checkout`, a cache restore.
- A single invocation, however long its argument list.
- A handful of straight-line commands.
- **One guard around an early exit.** `if [ … ]; then echo "::error::…"; exit 1; fi` is a
  precondition, not dispatch, and reads correctly at a glance.

It is a pragmatic scanner, not a shell parser: it flags the high-signal markers of "this is a
program" and favors precision over recall. A borderline body that slips through is still worth
extracting — an extracted script is testable, and this one isn't.

### Where the logic should go

Into a tested package in the repository's own language, split the way the
[composition-root pattern](/explanation/isolation) asks: a pure decision layer holding the
branching, and a thin entry point that supplies the real I/O. The workflow then invokes it through
a declared bin, and the `run:` line is one call.

Pass data in through the step's `env:` rather than interpolating `${{ }}` into the script body. An
`env:` value is templated safely; inline interpolation splices attacker-controlled text straight
into the shell.

## Which files are scanned

The YAML GitHub actually executes, and nothing else:

- every `*.yml` / `*.yaml` **directly inside** a `workflows/` directory;
- every file named `action.yml` / `action.yaml`, at any depth — a composite action's
  `runs.steps` is scanned exactly like a job's.

Discovery follows GitHub's own rules rather than taking every `*.yml` in the tree, so a lockfile or
a test fixture that happens to sit under `.github/` is never mistaken for CI. A named file is
always scanned, so you can point the check anywhere.

A document that does not parse yields no findings rather than an error. Malformed workflow YAML is
[actionlint](https://github.com/rhysd/actionlint)'s to report, and it does so far better than this
check could.

## When it runs

Always, as the `Workflow lint` job — the files it reads are in the tree on every event, so there is
nothing to scope to a diff. The job scans the calling repository's `.github`, the subcommand's
default, whatever `source` the call names: the CI under test is the repository's, not the package's.
A repository with no workflows passes untouched.

Run it from the CLI:

```sh
npx testing-conventions workflow-lint          # defaults to .github
npx testing-conventions workflow-lint path/to/workflow.yml
```

## Configuration

The check reads no configuration and honors no exemption rules.

That is deliberate. The escape hatch for every other check is a per-file exemption with a reason,
because a file can be a genuine exception. A `run:` body cannot: the argument for leaving logic
inline is always that extracting it is not worth the trouble, and the whole point of the check is
that the trouble is one function and one test. A step that truly cannot move is a sign the work
belongs in a composite action or a real program, not in an exemption.

A consumer that needs the job off leaves it out of [`gates`](/reference/workflow#inputs) — the
allowlist is authoritative.

## Learn more

- [`packaging`](./packaging) — the other check that reads build output rather than source.
- [Respond to a red check](/guide/configure) — what to do when this one fires.
