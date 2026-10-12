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

The CLI's own surface is documented where it runs — a check's invocation on that check's page, and
the shared flags (`--language`, `--config`, `--base`) in one place, per **The CLI is documented
where it runs** below. A check's page names the flags unique to it, tutorials and how-tos stay on
the workflow, and a new shared flag lands in that one section rather than growing a page of its
own.

## The structure: CI-shaped, with the check as the leaf

The docs are organized around how a consumer meets this tool — as named jobs in a pull request's
check list. Three top-level pages carry everything:

| Page | Job |
| --- | --- |
| `/` | What this does, in one screen: what goes red, and why you want that. |
| `/integrate` | Adopting it — the drop-in, one call per package, every input, the config file, the CLI's shared flags, and the author-side commands. |
| `/checks/` | Every check, grouped by category, with an index page at each level. |

### The check tree

Category → sub-category → check, each level carrying its own index page:

```
/checks/                          every check, one table, grouped by category
  /checks/testing/
    /checks/testing/unit/         colocated-test, one-function-per-file, unit-lint,
                                  unit-coverage, mutation
    /checks/testing/integration/  integration-lint
    /checks/testing/e2e/          e2e-verify
  /checks/packaging/              packaging
  /checks/repo/                   changelog
```

Each check page is the complete record for that check and **opens with why it exists** — the
lint-rule-page genre of ESLint, Ruff and Clippy. That opening was Diátaxis's one sanctioned hybrid
here; it is now the rule. The page then names the job or jobs that run the check, its run
conditions, its per-language behavior, every config key and exemption rule name that touches it,
and its CLI invocation.

### The job shape is a lookup, so it is a table

**The check is the leaf, and the jobs are one flat table on `/checks/`.** Two job boundaries do
not survive the nesting: `Static checks` runs four unit checks *plus* `integration-lint`, so it
spans two sub-categories, and `unit-coverage` is a single check behind a single flag that produces
two jobs. Making the job the leaf costs the `unit/` and `integration/` levels outright. Reshaping
CI to match the tree splits a job whose whole purpose is sharing one checkout across five
sub-second scans, and renames branch-protection surface, to settle a docs question.

So `/checks/` carries one table mapping every job name a consumer can see in a pull request to its
category and the checks it runs. A reader arriving with a red job name gets one lookup, and a
reader browsing gets a hierarchy that nests cleanly. **A new job name joins that table in the same
PR that adds the job** — the table is where the job surface is written down.

### The CLI is documented where it runs

A check's invocation lives on that check's own page, because the CLI is how the check runs. One
`/integrate#cli` section carries the flags every check shares (`--language`, `--config`,
`--base`) and the author-side commands that have no job at all (`install`, `e2e attest`,
`e2e slug`).

### `agents` is not a category yet

`install` writes the testing contract into a repository's agent context file: a command, not a
check, and no job runs it. It lives on `/integrate` under "Teach your agent". `/checks/agents/`
opens when the first check belongs in it.

### Until the pages move

This section is the decision. The pages migrate in later steps, and the site builds green at every
one, so until a page's step lands:

- `reference/`, `explanation/` and `guide/` still exist, and **Diátaxis (retiring)** below governs
  edits to the pages still in them.
- A **new** page goes in the tree above rather than into a retiring directory.
- The check inventory is still enumerated by hand in four places — the sidebar,
  `reference/checks/index.md`, `reference/workflow.md` and `explanation/index.md` — so a new check
  is added to all four until the pages consuming `docs/checks.json` land. The source's form is
  settled; see **The inventory has one authored source** below.

### The inventory has one authored source

`docs/checks.json` is the inventory: every check's name, category, sub-category, the job(s) that run
it, its run conditions, its languages, and its CLI invocation. **A page or a sidebar that lists
checks reads it rather than restating it.**

**A file, not a `tc-checks` subcommand, because the docs build stays hermetic.**
`docs/.vitepress/config.ts` is evaluated at build time, so generating the sidebar from the CLI would
make `pnpm run build` depend on a built Rust binary and hand a docs-only pull request the Rust
toolchain. Today that build imports nothing external. Shelling out from a config file to populate a
config file also puts a program where nothing tests it.

**An authored file is a fifth hand-maintained list unless something holds it to the truth**, so a
check does. Three of the surfaces it names are facts rather than opinions — the `gates` input's
accepted values, the job names in `.github/workflows/testing-conventions.yml`, and the CLI's leaf
commands — and `checks.inventory.inventory_drift` reports any name present on one side and missing
from the other, in both directions. A check deleted from the tool cannot keep its page, and a check
added to the tool cannot stay undocumented.

The pages and the sidebar are *generated from* the manifest, so they are not surfaces to reconcile;
that is what makes one authored file safe. **Job names are stored exactly as the workflow writes
them**, matrix template and all, so the comparison is string equality with no normalization step to
get wrong.

Two groups exist because the reconciliation has to be total, and both earn a documented home:
`jobs_without_a_check` (today, `detect`, which computes the language matrix) and
`commands_without_a_job` (`install`, `e2e attest`, `e2e slug` — the author-side commands **The CLI
is documented where it runs** sends to `/integrate#cli`).

## Diátaxis (retiring): four modes, one job per page

This section governs the pages still under `reference/`, `explanation/` and `guide/`, and retires with the last of them.

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
proportions are: two tutorials, one guide, a Reference section that carries the lookup weight
(the workflow, the config schema, and one page per check), and an Explanation section that
carries the intellectual weight.

### What each mode must *not* do

- **Tutorial** — no exhaustive options, no "why", no alternatives. One path that works, with an
  observable result at every step. Anything beyond the happy path is a link. Keep it short; if it's
  growing, content is leaking in from another mode.
- **How-to** — task-focused steps, not a lecture. State the *why* in one line and link to an
  Explanation page; link to Reference for exact keys and schemas. A guide that spends more words
  explaining the concept than performing the task is really an Explanation page.
- **Reference** — describe what *is*, completely and without teaching. No tutorials, no opinions.
  It must be *correct first*: a stale reference is worse than none. **One sanctioned hybrid:** each
  check's page under `reference/checks/` opens with a "Why this check exists" section — the
  lint-rule-page genre (ESLint, Ruff, Clippy): a reader landing on a check's page gets its complete
  picture, motivation included. An issue number appears only where the design is incomprehensible
  without its history, never as decoration; everything else historical stays in `CHANGELOG.md`.
- **Explanation** — the "why". Concepts, design rationale, trade-offs, the bigger picture. No
  step-by-step instructions; link to the guide and reference instead. Each check essay keeps its
  compact, factual "What it enforces" section — authored here inside `#region` markers and
  included into the check's reference page, so the precision facts are written once and rendered
  in both places.

When a page wants to do two jobs, split it and cross-link. The mutation pages are the worked
example: [Explanation — Mutation](./explanation/mutation) carries the concept and the engines;
[Respond to a red check](./guide/configure) carries the exemption mechanics.

## Duplication is deliberate

A reader arrives with a perspective — following a tutorial, looking up a fact, seeking the why —
and wherever they land must be sufficient on its own. So the same fact appears on every page whose
reader needs it: a check's coverage keys live on the check's page *and* in the config schema; its
motivation opens the check's page *and* fills its essay. Exhaustive and duplicative beats
non-duplicative. Two disciplines keep the copies honest:

- **Single-source identical text.** A fragment that would be word-for-word identical in two places
  is authored once inside `<!-- #region name -->` / `<!-- #endregion name -->` markers and pulled
  into the other page with `<!--@include: <path>#name-->` — the pattern the README rules list
  already uses. Links inside a shared region are root-absolute (`/reference/config#coverage`,
  never `../reference/config`), so they resolve from every including page. Restate by hand only
  where the two renderings genuinely differ in voice or depth.
- **Sweep every statement.** A PR that changes a check's behavior updates every place that states
  it: the check's reference page, its essay, the workflow table, the config schema. The docs land
  in the same PR as the change, so the sweep is part of writing them, not a follow-up.

## Open with the why

Every page — in any mode — opens with one or two sentences answering *why am I here?*: what this
page is for, and why the reader should care or read on. No page starts cold with mechanics. This is
the single most important habit for keeping the docs scannable.

## Information architecture (retiring)

This describes the four-mode layout being replaced; **The structure** above is the target a new page is written into.

The **homepage (`index.md`) is the hub** — a jumping-off point that routes to the sections, and the
only page whose job is navigation. The nav and sidebar mirror the four modes, in this order; keep
them in sync with `.vitepress/config.ts`.

- **Tutorials** — `getting-started.md` (the single-package drop-in) and `monorepo.md` (one call per
  package). These are the two guaranteed adoption paths, and the only tutorials.
- **How-to Guides** (`/guide/`) — `configure.md`, the one guide: the reader's single recurring task
  is responding to a red check (relax a floor, exempt a file or line, with a reason).
- **Reference** (`/reference/`) — `workflow.md` (every input, every check and its run condition,
  the `@v0` contract), `config.md` (the TOML schema, every default, the shared test configs), and
  `checks/` — one page per check, each the complete record of that check: motivation,
  per-language behavior, run conditions, configuration surface, exemption rule names. A check's
  page is the single starting point for that check; the same facts also appear wherever another
  page's reader needs them (see **Duplication is deliberate**).
- **Explanation** (`/explanation/`) — `index.md` (the testing model) and the essays:
  `colocated-test`, `coverage`, `mutation`, `isolation`, `packaging`, `e2e`, plus `scoping` (the
  scoping/exemption design). Essays map to concepts, not checks 1:1 — `isolation` covers both
  lint checks, because they enforce one boundary from opposite sides; the two check pages under
  `reference/checks/` each link to it.

A new page picks its directory by mode, not by topic — and the default for a new page is *no new
page*: fewer, denser pages beat a page per feature.

## Language and terminology

Consistency is a feature: the same idea uses the same word everywhere. **check**, **rule**, and
**gate** carry one standard, stated in full at the [glossary](/reference/glossary) — that page
binds this file, `README.md`, and the root `AGENTS.md`, not only pages under `docs/`.

- **check** — one of the runnable units the workflow enforces (`colocated-test`,
  `unit-coverage`, `mutation`, `integration-lint`, …), each named by the value the [`gates`]
  input takes and that appears in a `rules = […]` exemption. A check is a unit of enforcement, not
  a CI job: some run as their own job, several share one job as steps, and one produces two jobs
  — see the glossary for the exact shape. Never "lint" for a check (a named lint rule like
  `no-first-party-mock` is a "rule", not a check) or "test".
- **rule** — an individual assertion inside a lint check (`no-monkeypatch`,
  `unmocked-collaborator`, …), never the check that runs it.
- **gate** — a check in its merge-blocking role, and the `gates` input. Use it there — "a binary
  gate, not a score", "an escape hatch needs a reason" — never as a loose synonym for check.
- **the drop-in** — the six-line reusable-workflow snippet a consumer adds. Not "the action" (it's a
  reusable workflow, not a composite action).
- **the scan root** — the directory the `source` input names; the only scoping mechanism.
- **floor** — a coverage threshold. Coverage has a *floor*; you don't "set coverage to 100".
- **first-party / external** — the isolation boundary. "External" = third-party packages *and*
  effectful standard-library APIs. Don't say "third-party" when you mean both.
- **the unit ladder** — colocated-test → coverage → mutation (exists → runs → verifies).
- **exemption** — the reason-required config escape hatch. Not "ignore", "skip", or "waiver".
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
