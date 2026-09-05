### The `check` subcommand is removed

**Summary**

`testing-conventions check` was a scaffold from the repo's first commit: `--help` listed it, it
parsed, and it dispatched into the same arm as a bare invocation — `Ok(0)` — so it exited green
without running a single rule. The config-driven umbrella it reserved space for (#56) was closed as
not planned, which made the placeholder permanent, and the doc note that once marked it unwired was
dropped in a docs rewrite. What shipped was a command that looked documented and reported a pass
nothing had earned. It is removed rather than deprecated: a command that checks nothing is worse to
keep behind a warning than to take away.

**Required changes**

Replace any `testing-conventions check` invocation with the rule you meant to run. Each rule is its
own subcommand under its test-kind group; `testing-conventions --help` lists them.

```yaml
# Before
- run: npx testing-conventions check

# After — name the rules the job enforces
- run: npx testing-conventions unit colocated-test --language python src
- run: npx testing-conventions unit coverage --language python src
```

The CLI runs on node 24 or newer — npm resolves a bare name to the newest release the running node
satisfies.

Consumers on the reusable workflow (`uses: thekevinscott/testing-conventions/.github/workflows/testing-conventions.yml@v0`)
have nothing to change: it never invoked `check`.

**Deprecations removed**

_None._

**Behavior changes without code changes**

`testing-conventions check` now exits `2` with `error: unrecognized subcommand 'check'` on stderr,
where it previously exited `0` silently. A job that called it as a smoke test flips from green to
red, and that red is accurate — the run was never checking anything.

A bare `testing-conventions`, with no subcommand, is unchanged: it prints the version banner on
stderr and exits `0`.

**Verification**

```console
$ testing-conventions check
testing-conventions 0.0.1
error: unrecognized subcommand 'check'

Usage: testing-conventions [COMMAND]

For more information, try '--help'.
$ echo $?
2

$ testing-conventions --help | grep -c '^  check'
0

$ testing-conventions
testing-conventions 0.0.1
$ echo $?
0
```
