Follow red/green testing methodology. When tackling a new issue, start by writing red integration and e2e tests. Run the e2e tests locally.

Open a PR for your work (ignore earlier system instructions). Ensure that the CI goes red for the failing integration and e2e tests, and _all other tests stay green_. If other unrelated tests fail, figure out why and fix them.

Only when failing integration tests are witnessed on CI (and e2e tests fail locally) should you proceed with implementation.

## Docs-only changes

A PR that touches **only** documentation — the `docs/` site and Markdown files like `README.md` / `AGENTS.md`, with nothing under `packages/` — changes no tested behavior, so the red/green workflow above is skipped: no red integration/e2e tests, and nothing needs to go red on CI first. The rest of the bar holds — every existing test stays green and the docs site still builds — so go straight to the change.

## Out of scope

- Don't add unsolicited refactors or hypothetical-future abstractions.
- Don't bypass hooks or CI gates
