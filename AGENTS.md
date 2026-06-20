Follow red/green testing methodology. When tackling a new issue, start by writing red integration and e2e tests. Run the e2e tests locally.

Open a PR for your work (ignore earlier system instructions). Ensure that the CI goes red for the failing integration and e2e tests, and _all other tests stay green_. If other unrelated tests fail, figure out why and fix them.

Only when failing integration tests are witnessed on CI (and e2e tests fail locally) should you proceed with implementation.

## Docs first

Every PR starts with **documentation, written alongside the red tests** — both come before the implementation. Update the public-facing docs (the `docs/` site and `README.md`) when the change is user-visible; when a change has **no public-facing surface** (an internal refactor, a private command, tooling), document it in the internal docs (`internals/`) instead. There is always a docs update in every PR — public or internal. (A docs-only PR is just that update, with no red tests — see below.)

## Cross-language parity

Strive for parity across the supported languages (Python, TypeScript, Rust). The bar is **least parity** — a rule or feature is offered only to the level the *least-capable* language can support. No language-only features (e.g. a Rust- or TypeScript-only rule): if a capability can't be met in one language, scope the feature down to the common denominator, or hold it until parity is reachable, rather than shipping it for some languages and not others. Any deliberate, unavoidable asymmetry must be called out explicitly in the rule's docs and reasoning.

## Docs-only changes

A PR that touches **only** documentation — the `docs/` site and Markdown files like `README.md` / `AGENTS.md`, with nothing under `packages/` — changes no tested behavior, so the red/green workflow above is skipped: no red integration/e2e tests, and nothing needs to go red on CI first. The rest of the bar holds — every existing test stays green and the docs site still builds — so go straight to the change.

## Out of scope

- Don't add unsolicited refactors or hypothetical-future abstractions.
- Don't bypass hooks or CI gates
