`workflow-lint scope [--base <ref>] [path]` prints `true` when `<base>...HEAD` changes a file
`workflow-lint` scans under `path` (default `.github`), and `false` otherwise. It uses the check's
own file test, exits 0 on every verdict, and prints `true` when there is no base or the diff cannot
be read.
