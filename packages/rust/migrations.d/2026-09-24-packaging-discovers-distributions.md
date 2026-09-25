### `packaging` discovers the distributions under a root

**Summary**

`packaging` took one distribution path and one required `--language`, so a caller holding a `dist/`
had to find the files itself and invoke the check once per file — a loop, a glob per extension, and
its own "nothing matched" error. The reusable workflow carried exactly that as 21 lines of shell in
a `run:` body.

The command now does the finding. A directory is searched recursively for `*.whl` / `*.tar.gz` /
`*.tgz` / `*.crate`, each distribution is checked under the language its extension names, and the
run reports how many it covered. A root holding none of the four fails and names them, so an empty
`dist/` reads as the misconfigured build it is rather than as a pass.

`--language` is now optional. It names the convention for a single path, which is how you check an
already-unpacked artifact root — a directory carries no extension to read — and how you check a
distribution named against its ecosystem's convention.

**Required changes**

_None._ Every existing invocation passes `--language` and behaves exactly as before.

**Deprecations removed**

_None._

**Behavior changes without code changes**

_None._ The reusable workflow still globs and loops; collapsing its `run:` body to one invocation
lands in a following release, once the published binary understands the new form.

**Verification**

Point it at a directory of built distributions:

```sh
npx testing-conventions packaging dist/
```

It names each distribution it checked, reports the count, and exits 0 when no test file shipped.
An empty directory exits 1 with `no recognized built distribution`.
