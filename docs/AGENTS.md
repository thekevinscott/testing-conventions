# Documentation conventions

These rules govern everything under `docs/`. They exist so the documentation stays navigable as it
grows: every page has one job, and a reader (human or agent) always knows where to look.

## Diátaxis: four modes, one job per page

The docs follow [Diátaxis](https://diataxis.fr/). Every page belongs to **exactly one** of four
modes, decided by what the reader needs *right now*. Mixing modes on a page is the failure this
structure prevents — it's how a tutorial silently grows into a reference.

| Mode | Reader's need | Voice | Lives in |
| --- | --- | --- | --- |
| **Tutorial** | "Get me started" — learning by doing | Imperative, one happy path, no choices | `getting-started.md` |
| **How-to guide** | "Help me do X" — a specific task | Imperative steps toward a goal | `guide/` |
| **Reference** | "What exactly is X" — look up a fact | Dry, complete, neutral; no teaching | `reference/` |
| **Explanation** | "Why is it this way" — understanding | Discursive prose; context and trade-offs | `explanation/` |

### What each mode must *not* do

- **Tutorial** — no exhaustive options, no "why", no alternatives. One path that works. Anything
  beyond the happy path is a link to a guide. Keep it short; if it's growing, content is leaking in
  from another mode.
- **How-to** — task-focused steps, not a lecture. State the *why* in one line and link to an
  Explanation page; link to Reference for exact flags and exit codes. A guide that spends more words
  explaining the concept than performing the task is really an Explanation page.
- **Reference** — describe what *is*, completely and without teaching. No tutorials, no opinions, no
  motivation beyond a one-line "why" where it aids lookup. Structure mirrors the software (one
  section per command). It must be *correct first*: a stale reference is worse than none.
- **Explanation** — the "why". Concepts, design rationale, trade-offs, the bigger picture. No
  step-by-step instructions and no command tables; those belong in How-to and Reference. Link out to
  them instead.

When a page wants to do two jobs, split it and cross-link. The mutation pages are the worked
example: [Explanation — Why mutation testing](./explanation/mutation) carries the concept;
[How-to — Run mutation testing](./guide/mutation) carries the commands.

## Open with the why

Every page — in any mode — opens with one or two sentences answering *why am I here?*: what this
page is for, and why the reader should care or read on. No page starts cold with mechanics. A how-to
states its goal (and which surface it tunes) before the steps; an explanation states the question it
answers; a reference section says what the command is for in a line. This is the single most
important habit for keeping the docs scannable.

## Information architecture

The **homepage (`index.md`) is the hub** — a jumping-off point that routes to the four sections, and
the only page whose job is navigation. Mode pages carry no navigation farms; in particular the
tutorial ends by pointing at the How-to section rather than listing every guide. The nav and sidebar
mirror the four modes, in this order; keep them in sync with `.vitepress/config.ts`.

- **Getting Started** (Tutorial) — the single tutorial page. One tutorial is fine: the four modes
  are equally *important*, not equally *sized*, and tutorials are normally the fewest.
- **How-to Guides** (`/guide/`) — one page per task, ordered by **first need** (what a new adopter
  hits first — customizing a floor, exempting a file), with advanced and opt-out paths last.
  `guide/index.md` is the section landing.
- **Reference** (`/reference/`) — `index.md` (the CLI + config) and `defaults.md`.
- **Explanation** (`/explanation/`) — `index.md` (the testing model) and one page per concept.

A new page picks its directory by mode, not by topic. "Mutation" is not a section — it's a concept
page in Explanation *and* a task page in How-to.

## Language and terminology

Consistency is a feature: the same idea uses the same word everywhere.

- **rule** — a single CLI check that fails CI on a violation (`unit coverage`, `integration lint`).
  Not "check", "lint" (except a named lint like `no-first-party-mock`), or "test".
- **the drop-in** — the six-line reusable-workflow snippet a consumer adds. Not "the action" (it's a
  reusable workflow, not a composite action).
- **floor** — a coverage threshold. Coverage has a *floor*; you don't "set coverage to 100".
- **first-party / external** — the isolation boundary. "External" = third-party packages *and*
  effectful standard-library APIs. Don't say "third-party" when you mean both.
- **the unit ladder** — colocated-test → coverage → mutation (exists → runs → verifies).
- **exemption** — the reason-required config escape hatch. Not "ignore", "skip", or "waiver" (except
  `integration lint` waivers, which the reference names precisely).
- **language names** — `Python`, `TypeScript`, `Rust` capitalized in prose; lowercase
  `python` / `typescript` / `rust` only as literal CLI `--language` values or config keys.
- **the three configuration surfaces** — keep them distinct and name each by what it tunes. The
  **config file** (`testing-conventions.toml`) is *what the rules enforce* — floors, exemptions. The
  **workflow inputs** (the action's `with:` block) are *where and how a run is scoped* — languages,
  path, base; call these "inputs" or "options", never "configuration". The **shared test config**
  (`vitestConfig` / the pytest plugin) mirrors the floor into the consumer's *own test runner*.
  Reserve the verb "configure" for the config file.

Voice:

- Second person, present tense, active. "The rule fails the build", not "the build will be failed".
- Imperative in tutorials and how-tos ("Add one file", "Run it over `src`").
- One canonical phrasing per fact. The strict default is **"a 100% floor"**; mutation is **"a
  binary gate, not a score"**. Reuse those exact phrasings rather than re-inventing them per page.
- Cross-link generously, but each link earns its place: How-to → Explanation for *why*, How-to →
  Reference for *exact*, Explanation → How-to for *do it*.

## Authoring workflow

- A docs-only change (only `docs/` and root Markdown, nothing under `packages/`) skips the
  red/green workflow in the root `AGENTS.md` — see its "Docs-only changes" section. The site must
  still build (`pnpm --dir docs build`).
- Single source of truth: the rule list is authored once in `README.md` (`#region rules`) and pulled
  into `index.md` via VitePress `@include`. Don't duplicate it.
- The `README.md` is the project's own front page and may carry more than one mode at once
  (a readme is a hybrid by nature); these per-mode rules govern the `docs/` site, which doesn't have
  that excuse.
