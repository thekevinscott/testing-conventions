# Documentation conventions

These rules govern everything under `docs/`. They exist so the documentation stays navigable as it
grows: every page has one job, and a reader (human or agent) always knows where to look.

## The reader adopts the workflow

The docs are written for one reader: a maintainer (or their agent) adopting the reusable workflow
via YAML. Every page assumes the workflow runs the rules; the CLI appears only where a human
genuinely runs it (`e2e attest`, `install`). The docs present **two choices** — adopt the standard
whole, or record a reasoned exemption — and stop there: escape hatches (the `gates` allowlist,
driving the CLI in another CI system) are documented as facts in Reference, and no tutorial or
guide promotes them as destinations.

## Diátaxis: four modes, one job per page

The docs follow [Diátaxis](https://diataxis.fr/). Every page belongs to **exactly one** of four
modes, decided by what the reader needs *right now*. Mixing modes on a page is the failure this
structure prevents — it's how a tutorial silently grows into a reference.

| Mode | Reader's need | Voice | Lives in |
| --- | --- | --- | --- |
| **Tutorial** | "Get me started" — learning by doing | Imperative, one happy path, no choices | `getting-started.md`, `monorepo.md` |
| **How-to guide** | "Help me do X" — a specific task | Imperative steps toward a goal | `guide/` |
| **Reference** | "What exactly is X" — look up a fact | Dry, complete, neutral; no teaching | `reference/` |
| **Explanation** | "Why is it this way" — understanding | Discursive prose; context and trade-offs | `explanation/` |

The four modes are equally *important*, not equally *sized* — and for this tool the honest
proportions are: two tutorials, one guide, two reference pages, and an Explanation section that
carries the intellectual weight (one page per check).

### What each mode must *not* do

- **Tutorial** — no exhaustive options, no "why", no alternatives. One path that works, with an
  observable result at every step. Anything beyond the happy path is a link. Keep it short; if it's
  growing, content is leaking in from another mode.
- **How-to** — task-focused steps, not a lecture. State the *why* in one line and link to an
  Explanation page; link to Reference for exact keys and schemas. A guide that spends more words
  explaining the concept than performing the task is really an Explanation page.
- **Reference** — describe what *is*, completely and without teaching. No tutorials, no opinions, no
  motivation beyond a one-line "why" where it aids lookup. It must be *correct first*: a stale
  reference is worse than none. Issue numbers and history stay in `CHANGELOG.md`, never here.
- **Explanation** — the "why". Concepts, design rationale, trade-offs, the bigger picture. No
  step-by-step instructions; link to the guide and reference instead. **One sanctioned hybrid:**
  each check's page carries a compact, factual "What it enforces" section (per-language behavior)
  before the why — the checks have no separate per-command reference, so the precision facts live
  on the check's own page, kept dry and clearly sectioned.

When a page wants to do two jobs, split it and cross-link. The mutation pages are the worked
example: [Explanation — Mutation](./explanation/mutation) carries the concept and the engines;
[Configure the rules](./guide/configure) carries the exemption mechanics.

## Open with the why

Every page — in any mode — opens with one or two sentences answering *why am I here?*: what this
page is for, and why the reader should care or read on. No page starts cold with mechanics. This is
the single most important habit for keeping the docs scannable.

## Information architecture

The **homepage (`index.md`) is the hub** — a jumping-off point that routes to the sections, and the
only page whose job is navigation. The nav and sidebar mirror the four modes, in this order; keep
them in sync with `.vitepress/config.ts`.

- **Tutorials** — `getting-started.md` (the single-package drop-in) and `monorepo.md` (one call per
  package). These are the two guaranteed adoption paths, and the only tutorials.
- **How-to Guides** (`/guide/`) — `configure.md`, the one guide: the reader's single recurring task
  is responding to a red check (relax a floor, exempt a file or line, with a reason).
- **Reference** (`/reference/`) — `workflow.md` (every input, every check and its run condition,
  the `@v0` contract) and `config.md` (the TOML schema, every default, the shared test configs).
  Every public fact lives on exactly one of these two pages; a fact stated anywhere else links
  here.
- **Explanation** (`/explanation/`) — `index.md` (the testing model) and one page per check:
  `colocated-test`, `coverage`, `mutation`, `isolation`, `packaging`, `e2e`, plus `scoping` (the
  scoping/exemption design). This is where the docs invest.

A new page picks its directory by mode, not by topic — and the default for a new page is *no new
page*: fewer, denser pages beat a page per feature.

## Language and terminology

Consistency is a feature: the same idea uses the same word everywhere.

- **check** — one of the seven CI jobs the workflow runs (`unit coverage`, `integration lint`),
  named as it appears in the pull-request UI. "Rule" is acceptable for the enforced convention
  itself; never "gate" (except the `gates` input, which the reference names precisely), "lint"
  (except a named lint like `no-first-party-mock`), or "test".
- **the drop-in** — the six-line reusable-workflow snippet a consumer adds. Not "the action" (it's a
  reusable workflow, not a composite action).
- **the scan root** — the directory the `source` input names; the only scoping mechanism.
- **floor** — a coverage threshold. Coverage has a *floor*; you don't "set coverage to 100".
- **first-party / external** — the isolation boundary. "External" = third-party packages *and*
  effectful standard-library APIs. Don't say "third-party" when you mean both.
- **the unit ladder** — colocated-test → coverage → mutation (exists → runs → verifies).
- **exemption** — the reason-required config escape hatch. Not "ignore", "skip", or "waiver"
  (except `integration lint` waivers, which keep that name).
- **language names** — `Python`, `TypeScript`, `Rust` capitalized in prose; lowercase
  `python` / `typescript` / `rust` only as literal `languages` values or config keys.
- **the two configuration surfaces** — the **config file** (`testing-conventions.toml`) is *what
  the rules enforce* — floors, exemptions; the **workflow inputs** (the `with:` block) are *where
  and how a run is scoped* — languages, path, base; call these "inputs", never "configuration".
  Reserve the verb "configure" for the config file. (The shared test configs mirror the floor into
  the consumer's own runner; they're a reference fact, not a surface pages steer readers toward.)

Voice:

- Second person, present tense, active. "The check fails the build", not "the build will be failed".
- Imperative in tutorials and how-tos ("Add one file", "Point `source` at `src`").
- One canonical phrasing per fact. The strict default is **"a 100% floor"**; mutation is **"a
  binary gate, not a score"**; scoping is **"`source` scopes the scan; exemptions name the
  omissions"**. Reuse those exact phrasings rather than re-inventing them per page.
- Cross-link generously, but each link earns its place: How-to → Explanation for *why*, How-to →
  Reference for *exact*, Explanation → How-to for *do it*.
- **Don't justify, just state.** Describe what something does and how to use it — not why it had to
  be built this way, or how it compares to an alternative the reader didn't ask about. Genuine
  design rationale belongs in Explanation, never in a how-to or reference.

## Authoring workflow

- A docs-only change (only `docs/` and root Markdown, nothing under `packages/`) skips the
  red/green workflow in the root `AGENTS.md` — see its "Docs-only changes" section. The site must
  still build (`pnpm --dir docs build`).
- Single source of truth: the rule list is authored once in `README.md` (`#region rules`) and pulled
  into `index.md` via VitePress `@include`. Don't duplicate it.
- The `README.md` is the project's own front page and may carry more than one mode at once
  (a readme is a hybrid by nature); these per-mode rules govern the `docs/` site, which doesn't have
  that excuse.

## Agent-facing digest (`llms.txt`)

The build emits an agent-facing entry point next to the HTML — `llms.txt` (a link-rich index of
every page) and `llms-full.txt` (the whole site concatenated as one markdown file), per the
[llmstxt.org](https://llmstxt.org) standard. It's **generated from these same pages** by
`vitepress-plugin-llms` at build time (configured in `.vitepress/config.ts`), so the digest tracks
the docs with no second corpus to hand-maintain or let drift — the pages stay the single source of
truth. Its audience is agents *using the shipped tool*, which is why this file (`docs/AGENTS.md`, the
authoring conventions for *contributors*) is excluded from it via `ignoreFiles`.

Two authoring consequences:

- **Every page carries a one-line `description` frontmatter.** It feeds both the page's HTML `<meta>`
  description and its one-line entry in `llms.txt`, so write it as a crisp distillation of the page's
  "why" opening — same voice and canonical terminology. A page without one still lists in `llms.txt`,
  just without its summary line, so don't skip it.
- **Nothing to commit.** The artifacts land in the git-ignored `.vitepress/dist`; CI (`docs.yml`)
  regenerates and deploys them on every push. Verify locally with `pnpm --dir docs build`, then read
  `docs/.vitepress/dist/llms.txt`.
