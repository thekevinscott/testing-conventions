**Fixed** Every job in the reusable workflow resolves the CLI from a runner-owned temp prefix
(`npm --prefix "$RUNNER_TEMP" exec`), so the version it runs comes from the registry and the
`version` input. The suite-executing jobs install the consumer's dependencies before invoking the
CLI, and `npx <name>` prefers a binary already in `node_modules/.bin`, so a repo carrying
`testing-conventions` as a devDependency ran its own pinned copy in those jobs and a registry copy
in the rest. **BREAKING** for a repo that pinned an older release that way — see
`../migrations.d/2026-09-01-cli-resolves-outside-the-checkout.md`.
