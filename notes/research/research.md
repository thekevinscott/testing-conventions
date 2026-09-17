# Research: what the evidence says about testing LLM-authored code

Synthesis of the sources captured under `sources/`, mapped onto the rules this library
ships. The goal it serves is the one in `goals.md`: a definition of the optimal testing
strategy for LLM-authored open-source code, grounded in empirical, peer-reviewed research.

Every claim below cites a captured source by its `sources/<slug>` directory. Where the
capture is abstract-only or blocked, the claim is marked as such. Where the evidence
contradicts or fails to support a rule, that is said plainly. Practitioner essays are
included because they shape how the field thinks, but they are graded as opinion.

Two earlier documents sit next to this one. `TESTING-PHILOSOPHIES.md` is a survey
organized by school of thought. `TESTING-STRATEGY-DISCUSSION.md` is a design log from
a different project and does not state this library's goals. Neither is authoritative.
Section 8 lists claims from those documents that the captured corpus does not support.

## Contents

1. Evidence grades
2. The library's rules, in one paragraph
3. Findings, by rule
   - 3.1 Coverage floor
   - 3.2 Mutation gate
   - 3.3 Unit isolation and mocking
   - 3.4 Colocated tests and one function per file
   - 3.5 Integration tier: no first-party mocks, no monkeypatch
   - 3.6 End-to-end tier outside CI
   - 3.7 Determinism and flakiness
   - 3.8 Security of generated code
4. What the evidence says about LLM-authored tests specifically
5. Tensions between the evidence and the rules
6. Gaps: what the rules assume that nothing here shows
7. Source ledger
8. Claims in the earlier documents that the corpus does not support

## 1. Evidence grades

| Grade | Meaning | Examples in the corpus |
|---|---|---|
| A | Peer-reviewed empirical study, full text captured | Inozemtseva ICSE 2014; Spadini MSR 2017 / EMSE 2019; Xiao EMSE 2024; Lam ISSTA 2019; Nagappan EMSE 2008; PGS (2506.18315); metamorphic prompt testing (2406.06864) |
| B | Peer-reviewed or venue-accepted, abstract only captured | Most `arxiv.org-abs-*` entries; Just FSE 2014 and QuickCheck are metadata only |
| C | Industrial deployment report from the vendor | Meta TestGen-LLM and ACH; IBM ASTER; SWE book chapters 13 and 23 |
| D | Position or definitional paper | Harden and Catch (2504.16472); Assured LLMSE (2402.04380); SandboxEval |
| E | Practitioner opinion | Fowler, Dodds, Schlawack, Bazel docs, Dan Luu |
| X | Excluded | Emergent Mind aggregator page (`relevant: false`); dirsql config page (tooling, not evidence) |

Grade B is the weakest link. Twenty of the arXiv sources are abstract-only captures. An
abstract states the headline number and hides the sample, the confound, and the threats to
validity. Numbers from grade B sources are reported here as the abstract states them and
should be read as unverified.

## 2. The library's rules, in one paragraph

Three kinds of test. Unit tests mock every collaborator and live in a file colocated with the
one source file they cover. Integration tests run real first-party code with only external
dependencies faked, and are linted against first-party mocks, monkeypatching, inline
patching, environment mutation, and constant patching. End-to-end tests mock nothing and run
outside CI, with their local run attested and verified. Two mechanical gates sit over the
unit tier: a coverage floor scoped to the diff, and a diff-scoped mutation gate that fails on
any unexplained surviving mutant. One function per file keeps each unit small enough that
these gates are meaningful per file.

## 3. Findings, by rule

### 3.1 Coverage floor

**Rule.** `unit coverage` enforces a floor on lines covered, scoped to the diff.

**What the evidence says.** Coverage is a weak proxy for fault detection once suite size is
controlled. That is the single best-replicated finding in the corpus.

- Inozemtseva and Holmes generated 31,000 suites over five Java systems up to 724 KSLOC
  (grade A, `www.cs.ubc.ca--rtholmes-papers-icse_2014_inozemtseva.pdf`). With size ignored,
  Kendall's tau between coverage and mutation-measured effectiveness ranged 0.50 to 0.83 on
  four projects and was negative on HSQLDB. Controlling for suite size "always lowered the
  correlation"; Joda Time dropped "to essentially zero." Stronger criteria (decision, MC/DC)
  added nothing over statement coverage. The authors' conclusion: coverage is "useful for
  identifying under-tested parts of a program" but "should not be used as a quality target."
- MUTGEN reports suites with "100% coverage but only 4% mutation score" on 204 subjects
  (grade B, `arxiv.org-abs-2506.02954`).
- Harman et al. restate the consensus: "increasing coverage does not provide a guarantee of
  increasing fault revelation" (grade D, `arxiv.org-pdf-2504.16472`).
- Dan Luu, from practice: line coverage is easy to drive to 100% while missing the bugs that
  matter, and random generation found bugs hand-written high-coverage suites did not (grade
  E, `archived/links-9-10-26.md`).
- Dodds argues against 100% coverage mandates but flags his own 70% figure as invented
  (grade E, `kentcdodds.com-blog-write-tests`).

**Reading for the library.** The evidence supports coverage as a *detector of untested
code*, which is exactly what a diff-scoped floor does: it refuses a change that lands lines
no test executes. It does not support coverage as a quality signal, and the library does not
use it as one. The rule's framing should stay "a floor that finds untouched code," never "a
target." The Inozemtseva result also warns that raising the floor buys little: the
correlation flattens above moderate coverage, and the paper notes generated suites rarely
reached very high coverage, so the high-coverage regime is under-studied.

Inozemtseva measured effectiveness by mutants, and up to 35% of mutants in one subject were
assumed equivalent. The paper is a correlation study and claims no causality.

### 3.2 Mutation gate

**Rule.** `unit mutation` is binary and diff-scoped. Any surviving mutant without a recorded
explanation fails the gate.

**What the evidence says.** Mutation is the only measure in the corpus that industrial
LLM-test pipelines have adopted as an acceptance criterion, and it is the countermeasure the
LLM-test literature converges on.

- Meta's ACH accepts a generated test only when it kills a targeted mutant. Deployed over
  10,795 Kotlin classes, generating 9,095 mutants and 571 hardening tests; engineers accepted
  73% of the tests (grade B abstract, `arxiv.org-abs-2501.12862`; grade C blog,
  `engineering.fb.com-...-meta-ach`). The blog states the design reason: "increasing coverage
  doesn't necessarily find faults."
- Meta's earlier TestGen-LLM gated on build, reliable pass across reruns, and measurable
  coverage gain. Of generated tests, 75% built, 57% passed reliably, 25% increased coverage;
  73% of recommendations were accepted (grade B, `arxiv.org-abs-2402.09171`). Assured LLMSE
  generalizes this as "never trust the LLM; verify the artifact" (grade D,
  `arxiv.org-abs-2402.04380`).
- LLM-generated mutants detect real faults at 87.98% versus 41.64% for rule-based mutants on
  851 real Java bugs, at the cost of 26.60 points more non-compilable mutants (grade B,
  `arxiv.org-abs-2406.09843`). This bears on a future choice of mutation tool; the library
  currently drives conventional mutators.
- Just et al. (FSE 2014) is the paper usually cited for "mutants are a valid substitute for
  real faults." Its capture is metadata only (`dl.acm.org-doi-10.1145-2635868.2635929`). The
  claim is well known but is not evidenced in this corpus.
- Harden and Catch separates *hardening* tests (guard against future regressions) from
  *catching* tests (catch a fault in the change itself), and argues coverage-driven
  generation falls into a "Regression Only Trap" (grade D, `arxiv.org-pdf-2504.16472`). It
  proposes recall-at-precision with p = 0.8 as the deployment bar: engineers tolerate one
  false alarm per four true catches.

**Reading for the library.** The gate's *binary, diff-scoped* shape matches ACH more closely
than it matches a mutation-score threshold. ACH does not compute a score; it demands each
targeted mutant die. The library demands every mutant in the diff die or be explained. Both
are per-mutant, not percentage, decisions. The library's "explained surviving mutant" is the
manual analogue of ACH's equivalent-mutant classifier, which reached precision 0.79 and
recall 0.47 (0.95 and 0.96 with pre-processing).

The cost side is thin. No captured source measures mutation-gate runtime in CI, the
false-alarm rate from equivalent mutants under a diff-scoped gate, or how often an "explained
mutant" annotation is honest. Harden and Catch's 0.8 precision bar is a proposal, not a
measurement.

### 3.3 Unit isolation and mocking

**Rule.** `unit lint` requires every collaborator mocked in a unit test, with typed mocks in
TypeScript.

**What the evidence says.** This is where the rules and the evidence are in most tension.
The corpus does not contain a study showing that mock-everything unit tests find more bugs.
It contains several sources warning of costs.

- Google's SWE book (grade C, `abseil.io-resources-swe-book-html-ch13.html`): mocking
  frameworks "seemed like a hammer fit for every nail," and the tests "required constant
  effort to maintain while rarely finding bugs." The stated preference order is real
  implementation, then fake, then stub, with interaction testing last. Mockist testing is
  "difficult to scale" at Google. This is a vendor retrospective with no numbers.
- Spadini et al. (grade A, `sback.it-publications-msr2017b.pdf` and
  `link.springer.com-article-10.1007-s10664-018-9663-0`): across four Mockito-using Java
  systems, 6.71% of 38,313 test dependencies were mocked. Databases (72%), web services (69%),
  and external dependencies (68%) were mocked; domain objects were mocked 36% of the time and
  Java libraries 7%. Developers report that "maintaining the behavior of the mock compatible
  with the behavior of original class is hard" and that "mocking increases the coupling
  between the test and the production code." Half of mock changes were forced by production
  code changes. A Mockito core developer called many mocks in one test "a big red flag" for
  design. One interviewee: "I do not remember a single case in which I found a bug using
  mocks."
- Xiao et al. (grade A, `link.springer.com-article-10.1007-s10664-023-10410-y`): 66% of 193
  Apache Java projects use a mocking framework, but selectively. In 70% of projects fewer
  than 10% of test files mock anything; 61.1% of mocked objects isolate external libraries.
- Over-mocking by agents (grade B, `arxiv.org-abs-2602.00409`): across 1.2 million 2025
  commits in 2,168 TypeScript, JavaScript, and Python repositories, 36% of agent test commits
  add mocks versus 26% of human ones. The abstract labels this "over-mocked" but the captured
  text reports a rate difference, not a fault-detection outcome.
- Practitioner consensus (grade E): Fowler's classical versus mockist distinction
  (`martinfowler.com-articles-mocksArentStubs.html`); Schlawack's "don't mock what you don't
  own," wrap the third party in a façade and mock that
  (`hynek.me-articles-what-to-mock-in-5-mins`); Dodds' "stop mocking so much stuff" and "the
  more your tests resemble the way your software is used, the more confidence they can give
  you" (`kentcdodds.com-blog-write-tests`, `...-testing-implementation-details`,
  `...-stop-mocking-fetch`).

**Reading for the library.** The rule is defensible on grounds the corpus does support, but
they are not "mocked unit tests find bugs." They are:

1. **Determinism for an agent.** `goals.md` and the discussion log both put "deterministically
   enforceable, unambiguous" first. "Mock the awkward collaborators and not the domain ones"
   is the judgment call Spadini documents developers making inconsistently: 70% of mocked
   dependencies were also left un-mocked somewhere else in the same suite. "Mock all of them"
   is lintable; "mock the right ones" is not.
2. **A unit tier that cannot be the sole verifier.** The SWE book's "rarely finding bugs"
   and the interviewee's "never found a bug using mocks" are claims about mocked tests as the
   *primary* defense. The library places fault detection in the integration and e2e tiers,
   which forbid first-party mocks. The unit tier's job is per-function assertion under a
   mutation gate.
3. **The mutation gate answers the vacuity risk.** The specific failure the SWE book
   describes, a test that "verifies" a mock's canned answer, is a test that kills no mutant in
   the function under test. MUTGEN's 100%-coverage / 4%-mutation suites are what that looks
   like at scale.

What the corpus does not answer: whether a mock-everything unit tier plus a real-collaborator
integration tier finds more faults per maintenance hour than an integration-heavy suite
alone. Spadini's coupling finding still applies: a rename in a collaborator forces edits in
every unit test that mocks it. The library's colocation and diff-scoped gates keep that cost
local, but they do not remove it.

### 3.4 Colocated tests and one function per file

**Rule.** `unit colocated-test` requires a 1:1 source-to-test file mapping and co-change
under `--base`. `unit one-function-per-file` limits each source file to one function.

**What the evidence says.** No captured source studies file layout. The rules are
justifiable only indirectly.

- Spadini's evolution data: 83% of mocks exist from the test class's first version and
  production changes force test changes half the time. Co-change enforcement makes that
  forced change visible in the same diff rather than a later failure.
- Meta's TestGen-LLM operates per class and reports "improved 11.5% of all classes to which
  it was applied" (grade B). ACH is per class. Per-unit gates are the unit of account in the
  only deployed LLM-test systems in the corpus.
- The SWE book's "rarely finding bugs" complaint was about mock-heavy tests over large
  units. One function per file caps the collaborator count per unit, which caps the mock
  count per test.

**Reading for the library.** These are structural rules that make the other gates
computable per file. They are not evidenced as quality interventions in themselves, and
should not be described as such.

### 3.5 Integration tier: no first-party mocks, no monkeypatch

**Rule.** `integration lint` forbids mocking first-party code. Python hygiene lints forbid
`monkeypatch`, inline patching, environment mutation, and constant patching. `unknown-tier`
rejects tests that fit neither tier.

**What the evidence says.** This tier is where the corpus is most supportive.

- The SWE book's preference for real implementations (grade C) and Spadini's finding that
  developers reserve mocks for infrastructure (grade A) both describe this tier: first-party
  code real, external services faked.
- Fowler's Test Pyramid names an intermediate "subcutaneous" layer through the service API
  and calls high-level tests "a second line of test defense" (grade E,
  `martinfowler.com-bliki-TestPyramid.html`). Dodds' trophy makes this tier the largest
  (grade E).
- Hermeticity. Bazel's docs define a hermetic action as one insensitive to the host and
  network (grade E, `bazel.build-basics-hermeticity`). The SWE book reports moving Google
  Assistant's presubmit to a hermetic setup "cut the runtime by a factor of 14, with
  virtually no flakiness," and Takeout's sandboxed presubmit "prevented 95% of broken
  servers from bad configuration" (grade C, `abseil.io-resources-swe-book-html-ch23.html`).
- Environment mutation is a flakiness source. Lam et al. found 86% of flaky tests at
  Microsoft were flaky only in CI and 80% used more than one thread; time, randomness, async
  wait, and resource leaks were the case-study root causes (grade A,
  `mir.cs.illinois.edu-...-LamETAL19RootFinder.pdf`). A lint against environment mutation
  removes one of the shared-state channels those categories run through.
- Over-mocking by agents (grade B, `arxiv.org-abs-2602.00409`) is the direct motivation for
  a mechanical ban: the abstract's recommendation is to encode mock anti-patterns in agent
  configuration.

**Reading for the library.** The no-first-party-mock rule is the corpus's most consistent
recommendation, reduced to a lint. The Python hygiene lints have no direct study behind
them, but each targets a mechanism (global mutable state) that the flakiness literature
implicates.

One asymmetry to keep visible: the hygiene lints exist for Python only, because
`monkeypatch` is a pytest construct. The parity rule in `AGENTS.md` asks for that to be
called out, and it is not evidence-driven either way.

### 3.6 End-to-end tier outside CI

**Rule.** `e2e attest` records a local run; `e2e verify` checks the attestation in CI. E2E
tests mock nothing and do not run in CI.

**What the evidence says.** The corpus supports keeping slow and non-hermetic tests off the
presubmit path. It does not study attestation.

- SWE book ch. 23 (grade C): presubmit should run "only fast, reliable ones"; a presubmit
  pass implies "very high likelihood (95%+) of passing the rest." Moving Takeout's end-to-end
  tests to two-hourly post-submit "cut the 'culprit set' by 12 times." Its "CI Is Alerting"
  sidebar argues that a blanket "nobody commits unless CI is green" is "probably misguided"
  when the red is not actionable.
- Fowler (grade E): end-to-end tests are "brittle, expensive to write, and time consuming to
  run," with the caveat that if they are fast and reliable "lower-level tests aren't
  needed."
- Dan Luu (grade E): tests should be cheap enough that developers actually run them; the
  2026 update reports frameworks becoming too slow to use routinely.

**Reading for the library.** The evidence covers "keep them off presubmit." The attestation
mechanism is the library's own design for a question the corpus does not address: how to
prove an out-of-CI run happened. The CI-hermeticity invariant in `AGENTS.md` is a stronger
form of the same principle, and the SWE book's sandboxed-presubmit numbers are its closest
external support.

### 3.7 Determinism and flakiness

**Rules touched.** The mutation gate's binary outcome, the flaky-test tolerance in the
integration tier, and the goal that rules be "deterministically enforceable."

- Lam et al. (grade A): 4.6% of tests flaky across five Microsoft projects, affecting on
  average 27.4% of builds; distinct flaky tests are few but the builds they break are many;
  no correlation between count of flaky tests and count of builds they fail.
- ChatGPT non-determinism (grade B, `arxiv.org-abs-2308.02828`): 75.76% of CodeContests
  tasks had zero identical outputs across requests; temperature 0 reduces but does not remove
  it.
- SWE book ch. 23: flake classification and rerun policies are part of "accessible"
  feedback.
- TestGen-LLM's "passes reliably" gate is a rerun gate (grade B).

**Reading for the library.** The generator is non-deterministic, so the gates must not be.
A binary mutation gate and a lint are deterministic given the same inputs. The library's
gates are designed on that assumption. The corpus does not measure how often the library's
own mutation runs are flaky, which is a question worth answering with its own CI data.

### 3.8 Security of generated code

**No rule.** The corpus has a security thread that no gate covers.

- Copilot completions: about 40% of 1,689 programs across 89 CWE scenarios vulnerable
  (grade B, `arxiv.org-abs-2108.09293`).
- Users with an AI assistant wrote "significantly less secure code" and were more confident
  it was secure (grade B, `arxiv.org-abs-2211.03622`; no figures in the abstract).
- Recursive Criticism and Improvement prompting reduced weaknesses across GPT-3 to GPT-4 on
  150 prompts (grade B, `arxiv.org-abs-2407.07064`; no magnitude given).
- SandboxEval (grade D, `arxiv.org-pdf-2504.00018`) is about isolating untrusted code during
  execution, not about testing it.

**Reading for the library.** These findings are real but out of the library's scope as it
stands. A security lint or SAST step is a separate concern. Listed here so the gap is
explicit rather than forgotten.

## 4. What the evidence says about LLM-authored tests specifically

This is the part of the corpus that most directly motivates the library. The findings stack
in one direction: an LLM's own tests are a weak oracle for its own code, and the fix is an
independent, mechanical check.

**Generated tests pass on the code they were generated against, including when it is wrong.**

- Under semantic-altering changes, generated tests' pass rate fell from 100% to 66%, and
  "more than 99% of failing SAC tests pass on the original program while executing the
  modified region" (grade B, `arxiv.org-abs-2603.23443`; eight LLMs, 22,374 variants). The
  tests were coverage-adequate (79% line, 76% branch) and tracked the code, not the
  specification.
- On SAP HANA code guaranteed absent from training data, LLMs took shortcuts, favoring
  compilability over semantic checks; behavior differed on the in-training LevelDB (grade B,
  `arxiv.org-abs-2604.14437`; no numbers in the abstract).
- SWE-bench: 7.8% of patches counted as correct while failing the developer-written suite;
  29.6% of plausible patches diverge behaviorally from the ground truth; reported resolution
  rates inflated by 6.2 points (grade B, `arxiv.org-abs-2503.15223`).
- Test smells: Assertion Roulette and Magic Number Test dominate in 20,505 LLM-generated
  suites compared against 779,585 human tests (grade B, `arxiv.org-abs-2410.10628`).
- LLMs cannot self-correct reasoning without external feedback; intrinsic self-correction
  degrades results (grade B, `arxiv.org-abs-2310.01798`, ICLR 2024; no figures in the
  abstract).
- Reward hacking in code environments: 54 categories catalogued over 517 trajectories; best
  detector reached 63% (grade B, `arxiv.org-abs-2601.20103`). Agents do edit harnesses and
  special-case visible tests.

**What has been shown to help.**

- Mutation-kill acceptance (ACH, above).
- Property-based feedback. PGS reports 23.1% to 37.3% relative pass@1 gains over TDD-style
  example feedback, and on LiveCodeBench raised the repair rate from 46.6% with public tests
  to 75.9% with property feedback (grade A, `arxiv.org-html-2506.18315v1`). The paper names
  the failure it breaks: the "cycle of self-deception" where generated tests share the code's
  misunderstanding.
- Hybrid property plus example tests: 81.25% bug detection versus 68.75% for either alone
  (grade B, `arxiv.org-abs-2510.25297`), but on 16 HumanEval problems with one model, so
  suggestive at best.
- Metamorphic prompt testing: paraphrase the prompt, generate several programs, flag
  disagreement. Detects 75% of erroneous GPT-4 programs at an 8.6% false positive rate on
  HumanEval (grade A, `arxiv.org-html-2406.06864v1`; 24 erroneous programs total).
- Verified generation: Clover cross-checks code, docstring, and Dafny annotations and reports
  no incorrect program passing all checks (grade B, `arxiv.org-abs-2310.17807`). DafnyBench
  tracks the state of the art (grade A, `openreview.net-pdf-id-yBgTVWccIx`). Not applicable
  to Python or TypeScript as the library ships them.
- Static-analysis-guided generation (ASTER) raises coverage over EvoSuite and CodaMosa and
  developers preferred its tests, but the vendor names fault detection as future work
  (grade C, `research.ibm.com-blog-aster-llm-unit-testing`; grade B,
  `arxiv.org-abs-2409.03093`).

**Reading for the library.** The mutation gate is the one countermeasure here that the
library already ships. Property-based testing is the second-best-evidenced one and the
library does not touch it; see section 6.

## 5. Tensions between the evidence and the rules

1. **Mock-everything unit tests versus "rarely finding bugs."** Covered in 3.3. The rule is
   held up by determinism and by the mutation gate, not by the mocking literature. The
   library's docs should say that, and should not cite the SWE book as support for the unit
   tier.
2. **Coverage floor versus "not a quality target."** Covered in 3.1. Resolved by scope: the
   floor detects untested diff lines and stops. Any future move to raise the floor as a
   quality lever runs against Inozemtseva.
3. **Agents over-mock, and the unit tier mandates mocks.** The over-mocking study measured
   mock-adding commits, not bad tests, but its recommendation is to encode anti-patterns in
   agent config. The library does that for the integration tier. It should be clear that a
   unit test with every collaborator mocked is the intended shape there. The integration
   lint stops an agent from placing a mocked test in the integration tier, and
   `unknown-tier` (Python and TypeScript) stops it from placing one outside any tier.
4. **TDD's defect reduction is not about test-first ordering in this corpus.** Nagappan et
   al. report 40% to 90% lower pre-release defect density at 15% to 35% more initial time
   across four industrial teams (grade A). The authors call it "research in the typical," not
   a controlled experiment, and the time cost is a management estimate. The library does not
   enforce ordering; `AGENTS.md` asks for red tests first as process. The evidence is
   consistent with that but not specific to it.
5. **Meta accepts 73%; the library accepts 100% or fails.** ACH and TestGen-LLM present tests
   to engineers who accept about three in four. The library's gates are binary with no human
   in the loop. That is a design choice for a repo where the agent is the author and the
   human reviews the PR; the corpus does not evaluate an unattended binary gate.

## 6. Gaps: what the rules assume that nothing here shows

- **No study of the library's gate design as a whole.** Nothing measures a diff-scoped
  coverage floor plus a binary diff-scoped mutation gate plus tier lints, on agent-authored
  code, against any baseline. The closest are Meta's pipelines, which differ in every
  dimension listed in 5.5.
- **Property-based testing is unenforced.** It is the second-best-evidenced countermeasure
  in section 4, Dan Luu's strongest practitioner recommendation, and QuickCheck's original
  paper is in the corpus as metadata only. A rule that requires a property test per exported
  function, or a lint that recognizes Hypothesis and fast-check, would be the highest-value
  addition the evidence points at. Parity is reachable: Hypothesis, fast-check, and proptest
  exist for all three languages.
- **Mutation tool choice.** LLM-generated mutants outperform rule-based ones on fault
  coupling by 46 points (grade B). The library drives conventional mutators. Whether that
  matters under a binary gate is unmeasured.
- **Cost of the mutation gate.** No source measures CI time, equivalent-mutant false alarms,
  or annotation honesty under a per-mutant gate.
- **File layout rules.** No evidence at all; see 3.4.
- **Attestation.** No evidence; see 3.6.
- **Security.** No rule; see 3.8.
- **Rust.** Every empirical mocking study is Java-only, and none studies trait-injected
  doubles, which is how the library's Rust rules isolate collaborators. The Python hygiene
  lints have no analogue in the literature or in the other two languages.
- **Abstract-only captures.** Twenty grade B sources. The highest-value ones to upgrade to
  full text are 2603.23443, 2602.00409, 2503.15223, 2501.12862, and Just et al. FSE 2014.

## 7. Source ledger

Grade, slug, and the one claim this document takes from each.

| Grade | Source | Used for |
|---|---|---|
| A | `www.cs.ubc.ca--rtholmes-papers-icse_2014_inozemtseva.pdf` | Coverage weakly correlated with effectiveness when size controlled |
| A | `sback.it-publications-msr2017b.pdf` | What developers mock; coupling cost of mocks |
| A | `link.springer.com-article-10.1007-s10664-018-9663-0` | Mock evolution; production changes force mock changes |
| A | `link.springer.com-article-10.1007-s10664-023-10410-y` | Mocking prevalence in Apache; used selectively |
| A | `mir.cs.illinois.edu-winglam-publications-2019-LamETAL19RootFinder.pdf` | Flaky prevalence; CI-only flakiness; root causes |
| A | `www.microsoft.com-...-nagappan_tdd.pdf` | TDD defect density and time cost |
| A | `arxiv.org-html-2506.18315v1` | PBT feedback beats example-test feedback |
| A | `arxiv.org-html-2406.06864v1` | Metamorphic prompt testing detection and FP rates |
| A | `openreview.net-pdf-id-yBgTVWccIx` | Formal verification benchmark state |
| B | `arxiv.org-abs-2603.23443` | Generated tests track code, not behavior |
| B | `arxiv.org-abs-2604.14437` | Shortcuts on unseen code |
| B | `arxiv.org-abs-2503.15223` | SWE-bench plausible-but-wrong patches |
| B | `arxiv.org-abs-2602.00409` | Agents add mocks more often than humans |
| B | `arxiv.org-abs-2410.10628` | Test smells in LLM tests |
| B | `arxiv.org-abs-2310.01798` | No intrinsic self-correction |
| B | `arxiv.org-abs-2601.20103` | Reward hacking taxonomy and detection |
| B | `arxiv.org-abs-2501.12862` | ACH numbers |
| B | `arxiv.org-abs-2402.09171` | TestGen-LLM numbers |
| B | `arxiv.org-abs-2406.09843` | LLM mutants vs rule-based |
| B | `arxiv.org-abs-2506.02954` | 100% coverage, 4% mutation score |
| B | `arxiv.org-abs-2510.25297` | PBT plus EBT hybrid |
| B | `arxiv.org-abs-2308.02828` | LLM non-determinism |
| B | `arxiv.org-abs-2108.09293` | Copilot vulnerability rate |
| B | `arxiv.org-abs-2211.03622` | AI-assisted users less secure, more confident |
| B | `arxiv.org-abs-2407.07064` | Secure-code prompting |
| B | `arxiv.org-abs-2310.17807` | Clover verified generation |
| B | `arxiv.org-abs-2409.03093` | ASTER paper (abstract) |
| B | `dl.acm.org-doi-10.1145-2568225.2568271` | Inozemtseva ACM record (partial; use the UBC PDF) |
| B | `dl.acm.org-doi-10.1145-2635868.2635929` | Just et al. (metadata only; no claim taken) |
| B | `dl.acm.org-doi-10.1145-351240.351266` | QuickCheck (metadata only; no claim taken) |
| B | `www.researchgate.net-publication-343887704` | Assessing Mock Classes (metadata only; no claim taken) |
| B | `www.sciencedirect.com-science-article-abs-pii-S0164121226000683` | JSS 2026 prompting study (metadata only; no claim taken) |
| C | `abseil.io-resources-swe-book-html-ch13.html` | Real > fake > stub; mocks rarely find bugs |
| C | `abseil.io-resources-swe-book-html-ch23.html` | Presubmit fast/reliable only; hermetic 14x |
| C | `engineering.fb.com-...-meta-ach` | ACH design rationale |
| C | `research.ibm.com-blog-aster-llm-unit-testing` | ASTER claims; fault detection is future work |
| D | `arxiv.org-pdf-2504.16472` | Hardening vs catching; R@P 0.8 |
| D | `arxiv.org-abs-2402.04380` | Assured LLMSE principle |
| D | `arxiv.org-pdf-2504.00018` | Sandboxing untrusted code |
| E | `martinfowler.com-articles-mocksArentStubs.html` | Classical vs mockist |
| E | `martinfowler.com-bliki-TestDouble.html` | Double taxonomy |
| E | `martinfowler.com-bliki-TestPyramid.html` | Pyramid and its caveat |
| E | `kentcdodds.com-blog-write-tests` | Mostly integration; 70% is invented |
| E | `kentcdodds.com-blog-the-testing-trophy-and-testing-classifications` | Trophy shape |
| E | `kentcdodds.com-blog-testing-implementation-details` | Implementation-detail brittleness |
| E | `kentcdodds.com-blog-stop-mocking-fetch` | Mock at the network boundary |
| E | `hynek.me-articles-what-to-mock-in-5-mins` | Don't mock what you don't own |
| E | `bazel.build-basics-hermeticity` | Hermeticity definition |
| E | `archived/links-9-10-26.md` (Dan Luu, danluu.com/testing) | Random and property testing beat hand-written; coverage easy to game; not yet captured under `sources/` |
| X | `www.emergentmind.com-topics-secure-code-generation-methods` | Excluded, aggregator |
| X | `thekevinscott.github.io-dirsql-cli-config.html` | Tooling reference, not evidence |

## 8. Claims in the earlier documents that the corpus does not support

`TESTING-PHILOSOPHIES.md` states several figures that the captured sources do not contain.
They may be true, but they cannot be cited from this corpus.

- "~31% of passing patches rest on weak/insufficient test suites" and "~48% of 'resolved'
  instances are actually incorrect" for SWE-bench. The captured abstract of 2503.15223 says
  7.8%, 29.6%, 46.8% (divergent implementations, not incorrect ones), and 6.2 points of
  inflation. The 31% and 48% figures do not appear.
- "Assessing Mock Classes tracked 87,014 methods across 18 quality metrics" with "persistent
  maintainability degradation." The capture of `www.researchgate.net-publication-343887704`
  is Crossref metadata only, with no abstract.
- "Across 357 real faults in 5 projects (321 KLOC), mutant-detection correlated with
  real-fault detection independently of coverage" for Just et al. The capture is metadata
  only.
- "More mocks did not improve coverage" attributed to Xiao et al. The captured full text
  reports usage patterns and states the study "does not argue for or against mocking."
- Luo et al. FSE 2014 flaky-test root-cause percentages (async-wait 45%, concurrency 20%,
  test-order 12%). That paper is not in the corpus.
- "ICSE Most-Influential Paper (2024)" for Inozemtseva. Not in the capture.

`TESTING-STRATEGY-DISCUSSION.md` records goals for a different project (dirsql), including
a ban on all in-code mocks and a uniform network-isolation mechanism. Those are not this
library's goals and this document does not use them.
