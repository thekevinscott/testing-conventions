**Fixed** The `vitestConfig` base declares the `text` coverage reporter with `skipFull: false`,
so an agent-driven coverage run prints the full table at the 100% floor. Vitest 4 injects
`skipFull: true` into the text reporter when an agent environment variable is present
(`CLAUDECODE`, `CURSOR_AGENT`, ...); at this package's floor every node sits at 100%, so the
table printed as headers around nothing. A declared value wins over the injection. **BREAKING**
for a consumer that relied on vitest's default reporter list or declares its own — see
`../migrations.d/2026-09-04-vitest-base-text-reporter-skipfull.md`.
