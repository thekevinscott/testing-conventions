### `e2e verify`'s second question scopes to the acting branch's own receipt

**Summary**

Question 2 asked "did any file under `e2e-attestations/` change in `<base>...HEAD`?" — a PR could
satisfy it by touching a receipt belonging to an unrelated, already-merged branch, with no `attest`
run of its own. The question now reads only the acting branch's own file,
`e2e-attestations/<branch_slug>.json`. The acting branch is `--branch` when given, otherwise the
checked-out branch; with neither resolvable (a detached HEAD and no `--branch`), `verify` falls
back to the prior whole-directory question rather than failing every consumer outright.

**Required changes**

_None._ A same-repo checkout that leaves HEAD on a named branch needs no change. The reusable
workflow's checkout — `ref: ${{ github.event.pull_request.head.sha }}`, the only form that
resolves a fork's head branch safely — leaves HEAD detached, so it falls back to the
whole-directory question until the workflow is updated to pass `--branch` in a follow-up PR (the
CLI change and its workflow wiring cannot land in the same release).

**Deprecations removed**

_None._

**Behavior changes without code changes**

A receipt belonging to a different, already-merged branch no longer answers Question 2 for the
acting branch — editing it fails the check where it previously passed. A checkout with a named
branch (same-repo, non-detached) is scoped to that branch's own receipt with no flag needed.

**Verification**

```console
$ testing-conventions e2e verify --branch fix/my-change --base origin/main
```

Editing only `e2e-attestations/some-other-branch.json` now fails, naming the fix; writing
`e2e-attestations/fix-my-change.json` via `e2e attest` passes.
