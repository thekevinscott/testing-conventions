### The workflow-lint check ships in the CLI

**Summary**

"`run:` holds no logic" has been written policy across these repositories and machine-enforced in
none of them. actionlint and shellcheck catch bad refs and shell bugs; neither flags a loop, a
`case` dispatch, or `sed` munging in a `run:` body. The only enforcement anywhere was a repo-local
Python gate in `thekevinscott/template-lib` that every repository made from the template inherited
as a copy, free to drift. That rule set now ships in the published binary as
`testing-conventions workflow-lint [path]`.

The rule is deliberately narrow. It flags the high-signal markers of "this is a program" —
iteration, `case` dispatch, `awk`/`sed`, and a body longer than 12 command lines — and leaves
everything else alone, including a single guard around an early exit, which is a precondition
rather than dispatch. A keyword counts only as the first word of a line, so `before_hook` and
`casements=3` are untouched. The check reads no configuration and honors no exemptions: the
argument for leaving logic inline is always that extracting it is not worth the trouble, and the
trouble is one function and one test.

**Required changes**

_None._ The command is additive; existing subcommands are unchanged.

**Deprecations removed**

_None._

**Behavior changes without code changes**

_None._ The reusable workflow does not run the check yet — the job wiring lands in a following
release, once the published binary understands the subcommand.

**Verification**

Run it at a repository root:

```sh
npx testing-conventions workflow-lint
```

It scans `.github` by default and names each offending step with its file, line, and the reasons it
was flagged. A repository with no workflows exits 0.
