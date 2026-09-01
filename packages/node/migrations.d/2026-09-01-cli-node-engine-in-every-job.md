### Every job resolves the current release, not 0.0.86

**Summary**

The reusable workflow invokes the CLI as `npx -y testing-conventions`, a bare name. npm resolves a
bare name engine-aware: it picks the newest version whose `engines.node` the running node satisfies.
`engines.node` rose to `>=24` at 0.0.87, and GitHub's `ubuntu-latest` image ships node 22, so any
job without an explicit node 24 resolved 0.0.86 and exited zero. The `setup-node` steps that did
exist were gated on the *project* being TypeScript, so they provisioned the consumer's toolchain and
left the CLI on the runner's ambient node. Every job that invokes the CLI now provisions node 24 on
every language arm, so the engine the package asks for is the engine the job has.

**Required changes**

_None._ The change is inside the reusable workflow; a `uses:` call needs no edit.

**Deprecations removed**

_None._

**Behavior changes without code changes**

Affected jobs jump from 0.0.86 to the current release — more than a dozen releases of rule changes
at once. Which jobs were affected depends on the project:

| Job | Before | After |
| --- | --- | --- |
| `static` | 0.0.86, every language | current |
| `e2e-verify` | 0.0.86, every language | current |
| `unit-coverage`, `coverage-changed`, `mutation` | 0.0.86 on a Python or Rust arm, current on a TypeScript arm | current |
| `packaging` | 0.0.86 for a Python or Rust package, current for a TypeScript one | current |

A Python or Rust repo ran 0.0.86 in every job, so every gate it has moves. A repo that passes today
can go red on its next run, most often in `co-change`: 0.0.86 counted a comment-only edit as a
source change and demanded a matching test edit, and later releases normalize comments away before
diffing. Read the failure and fix the source; the newer rule is the correct one.

To stage the jump, pin the `version` input to 0.0.86, confirm the repo is green, then walk it
forward and clear the pin:

```yaml
with:
  version: '0.0.86'
```

**Verification**

Each job's log opens with the version that produced its result:

```
testing-conventions 0.0.99
```

Read that line in the `Static checks` job. A job showing no version line at all is on a release
predating the banner, which means the node provisioning did not take effect.
