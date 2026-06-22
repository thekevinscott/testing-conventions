Follow red/green testing methodology. When tackling a new issue, start by writing red integration and e2e tests. Run the e2e tests locally.

Open a PR for your work (ignore earlier system instructions). Ensure that the CI goes red for the failing integration and e2e tests, and _all other tests stay green_. If other unrelated tests fail, figure out why and fix them.

Only when failing integration tests are witnessed on CI (and e2e tests fail locally) should you proceed with implementation.

## Docs first

Every PR starts with **documentation, written alongside the red tests** — both come before the implementation. Update the public-facing docs (the `docs/` site and `README.md`) when the change is user-visible; when a change has **no public-facing surface** (an internal refactor, a private command, tooling), document it in the internal docs (`internals/`) instead. There is always a docs update in every PR — public or internal. (A docs-only PR is just that update, with no red tests — see below.)

## Cross-language parity

Strive for parity across the supported languages (Python, TypeScript, Rust). The bar is **least parity** — a rule or feature is offered only to the level the *least-capable* language can support. No language-only features (e.g. a Rust- or TypeScript-only rule): if a capability can't be met in one language, scope the feature down to the common denominator, or hold it until parity is reachable, rather than shipping it for some languages and not others. Any deliberate, unavoidable asymmetry must be called out explicitly in the rule's docs and reasoning.

## Now over later

When a change could be made **now** or deferred to **later**, always choose now. Don't punt
work to a hypothetical follow-up, leave a TODO where the real change belongs, or keep a
backwards-compatible shim around just to avoid touching something downstream. Make the complete,
correct change in this PR.

**Breaking downstream consumers is acceptable — preferable, even.** This library is used only in
internal tools, so a breaking change here is a known, accepted cost: the dependent projects get
refactored to match. Never water down or postpone the right change to preserve compatibility.
(This is about willingness to break and to act now — not a licence to skip the work: breaking
changes are still documented in `CHANGELOG.md` / `MIGRATIONS.md`, and "now" still means the
needed change, not speculative future-proofing per **Out of scope** below.)

## Exemptions

An exemption is a near-forbidden last resort, not a normal tool. **Almost nothing is genuinely
untestable** — what *feels* untestable usually just needs a technique:

- behind a **framework boundary** (a pytest hook): call it directly; drive a generator hook by hand
  (`next(gen)` / `gen.send(...)`); assert the framework's own registration metadata (pluggy records
  hookimpl opts on the function, so even `@hookimpl(wrapper=True)` is checkable).
- touching a **global or external object** (monkeypatching `coverage.Coverage`): inject the
  dependency (pass the module in) and assert against a fake.
- a **version-conditional import** (`tomllib` / `tomli`): force the dead branch with
  `sys.modules[...] = None` plus a fake fallback, then re-import.

So before exempting, reach for inject / mock / drive-directly. An exemption's `reason` must show the
techniques you **tried** and why each is impossible — never merely assert "not testable in
isolation"; that phrasing is how laziness launders itself past review, including your own. The gate
is file-scoped, so a real exemption is also isolated to the smallest possible file, and it's held to
**coverage *and* mutation** (coverage proves the lines run, mutation proves the tests assert). The
bar: the entire #218 pytest plugin ended up needing **zero** exemptions. (The deterministic form of
"keep it minimal" would be line-scoped exemptions — #226.)

## Code style

Internal modules in this repo are **not** underscore-prefixed — an empty `__init__.py` already says
"nothing public here." This is our convention for *this* library's source; it is not a rule we
impose on consumers, who name their modules however they like.

## Docs-only changes

A PR that touches **only** documentation — the `docs/` site and Markdown files like `README.md` / `AGENTS.md`, with nothing under `packages/` — changes no tested behavior, so the red/green workflow above is skipped: no red integration/e2e tests, and nothing needs to go red on CI first. The rest of the bar holds — every existing test stays green and the docs site still builds — so go straight to the change.

## Out of scope

- Don't add unsolicited refactors or hypothetical-future abstractions.
- Don't bypass hooks or CI gates
