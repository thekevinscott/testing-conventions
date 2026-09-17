# Testing Philosophies

A survey of testing philosophies, organized by philosophy. For each: what it
espouses, its adherents, pros/cons, and — separated out — the *rigorous*
evidence, with numbers where it exists. Claims are flagged where they rest on
real studies versus expert opinion, so it is clear what we can *rely* on.

> **Status — survey, not a decision.** This is a literature map meant to
> *inform* dirsql's testing strategy; it is **not** an adopted policy. The
> high-level goals, every attempt made, and the still-open decision are tracked
> in [`TESTING-STRATEGY-DISCUSSION.md`](./TESTING-STRATEGY-DISCUSSION.md).
> **No solution has been agreed.**

A note up front on evidence quality: the **mocking-school debate is argued
mostly through essays, not controlled experiments** — there is no clean RCT of
"mockist vs classicist." What *is* rigorously studied: coverage↔effectiveness,
mutation validity, flaky-test causes, mock prevalence/maintainability, and TDD
defect density. We lean on those hardest. For AI-authored code (§E) the
rigorous results are newer but real and growing fast: intrinsic self-correction
*degrades* accuracy, test-oracle leakage on agent patches is measured, AI code
carries elevated vulnerability rates, agent-generated tests are shown to be superficial rather than behavior-tracking, and assured mechanical gating is deployed
at industrial scale.

---

## A. Mocking philosophies

### 1. Classicist / "Detroit" / Chicago school (state-based)
- **Espouses:** Use *real* collaborators; verify final state/output; reach for a test double only when a dependency is genuinely awkward (slow, external, nondeterministic).
- **Adherents:** Kent Beck, Martin Fowler (sympathetic), "Uncle Bob" Martin; Google's institutional doctrine aligns.
- **Pros:** Higher fidelity, refactor-tolerant (tests don't know internal structure), fewer false greens.
- **Cons:** Slower (real deps), coarser failure localization (a bug fails many tests), needs real infra available.
- **Evidence:** Google's *Software Engineering at Google* reports the mockist style is "**difficult to scale**" and that overuse of mocks "got **out of sync with the real implementation** and made refactoring difficult," so they now prefer real implementations ([SWE Book, ch.13 — Test Doubles at Google](https://abseil.io/resources/swe-book/html/ch13.html)).

### 2. Mockist / "London" school (interaction-based)
- **Espouses:** Replace *all* collaborators with doubles; verify *interactions* (which calls happened); drive design outside-in.
- **Adherents:** Steve Freeman & Nat Pryce (*Growing Object-Oriented Software Guided by Tests*), J.B. Rainsberger.
- **Pros:** Precise failure localization, fast, isolates the unit, pressures you to define clean interfaces.
- **Cons:** Couples tests to implementation; resists refactoring; "mock drift" (the double diverges from reality); false confidence.
- **Evidence:** Foundational framing in Fowler, [*Mocks Aren't Stubs*](https://martinfowler.com/articles/mocksArentStubs.html). Empirically, Spadini et al. find developers report "**mocking increases the coupling between the test and the production code**" and that **keeping the mock's behavior compatible with the original is hard** ([*To Mock or Not To Mock?*, MSR 2017](https://sback.it/publications/msr2017b.pdf)). *Assessing Mock Classes* tracked **87,014 methods across 18 quality metrics** and found mock-impacted methods are "more complex, less readable, harder to maintain," with "**persistent maintainability degradation**" ([researchgate](https://www.researchgate.net/publication/343887704)).

### 3. "Don't mock what you don't own" / Fakes-over-mocks (the boundary position)
- **Espouses:** Don't fake third-party code directly; wrap it in a thin adapter *you* own and fake that. Prefer a **fake** (a real, in-memory implementation) over an interaction **mock**. Mock at the *system boundary* (e.g., the network), not your modules.
- **Adherents:** Google (real > fake > mock hierarchy, [ch.13](https://abseil.io/resources/swe-book/html/ch13.html)); Hynek Schlawack in Python ([*"Don't Mock What You Don't Own"*](https://hynek.me/articles/what-to-mock-in-5-mins/)); Kent C. Dodds via MSW ([*Stop Mocking Fetch*](https://kentcdodds.com/blog/stop-mocking-fetch) — mock the network boundary, "leaves your box intact").
- **Pros:** Resistant to refactoring and to library churn; fakes drift less than mocks; high fidelity.
- **Cons:** You must build and maintain the fake/adapter.
- **Evidence:** Spadini et al.: developers *do* converge here in practice — "classes that deal with **external resources, such as databases and web services, are often mocked**," while "**domain objects are usually not mocked**" ([MSR 2017](https://sback.it/publications/msr2017b.pdf); [EMSE 2019](https://link.springer.com/article/10.1007/s10664-018-9663-0)). Prevalence is high and rising: ~23% of 5,000 GitHub projects (2014) → **66% of Apache Java projects** ([EMSE 2023](https://link.springer.com/article/10.1007/s10664-023-10410-y)), where the same study notes **more mocks did not improve coverage**.

*Taxonomy note (Meszaros):* "test double" is the umbrella term; the five types are **dummy, stub, spy, mock, fake** — a *fake* "has a working implementation… an in-memory database is a good example" ([Fowler, *Test Double*](https://martinfowler.com/bliki/TestDouble.html), after Gerard Meszaros, *xUnit Test Patterns*). Much confusion is people saying "mock" for what is really a stub or fake.

---

## B. Suite-shape philosophies

### 4. The Test Pyramid
- **Espouses:** Many fast unit tests, fewer integration, very few slow E2E/UI.
- **Adherents:** Mike Cohn (coined it in *Succeeding with Agile*, 2009); popularized by Fowler ([*TestPyramid*](https://martinfowler.com/bliki/TestPyramid.html)).
- **Pros:** Fast feedback, cheap, stable.
- **Cons:** Unit-heavy suites can pass while the integrated system is broken; the cost assumptions predate cheap modern higher-level tooling.
- **Evidence:** Largely a heuristic/experience model, not an empirically-derived ratio — treat as guidance, not a measured optimum.

### 5. The Testing Trophy / "mostly integration"
- **Espouses:** Static analysis + types at the base, a **large integration layer**, smaller unit and E2E. Guiding rule: "**The more your tests resemble the way your software is used, the more confidence they can give you**"; corollary: **don't test implementation details.**
- **Adherents:** Kent C. Dodds ([*Write tests. Not too many. Mostly integration.*](https://kentcdodds.com/blog/write-tests); [*Testing Trophy*](https://kentcdodds.com/blog/the-testing-trophy-and-testing-classifications); [*Testing Implementation Details*](https://kentcdodds.com/blog/testing-implementation-details)); Testing-Library community.
- **Pros:** High confidence-per-effort; refactor-tolerant.
- **Cons:** Integration tests are slower and can be harder to debug; less granular localization.
- **Evidence:** Argued from experience + the confidence principle; no controlled study establishes the trophy ratio. Reliable part is the *anti-pattern* claim (testing implementation details → brittleness), which aligns with the mock-coupling evidence above.

---

## C. Isolation philosophy

### 6. Hermetic testing
- **Espouses:** A test is sealed from the outside world (no network/DB/services); it spins up its own throwaway deps; same result on any machine. Enforced by the **build system/sandbox**, not author discipline.
- **Adherents:** Google ([SWE Book ch.23](https://abseil.io/resources/swe-book/html/ch23.html)); Bazel/Buck/Pants ([Bazel — Hermeticity](https://bazel.build/basics/hermeticity)).
- **Pros:** Reproducibility ("your machine no longer matters"), kills a whole class of flakiness, enables remote caching.
- **Cons:** Real infrastructure cost (a hermetic build tool or sandbox).
- **Evidence:** Indirect but strong — the dominant flaky-test causes are exactly the external/nondeterministic effects hermeticity removes: in the canonical study, **async-wait 45%, concurrency 20%, test-order 12%**, plus network/time/I/O/randomness (Luo, Hariri, Eloussi, Marinov, *An Empirical Analysis of Flaky Tests*, FSE 2014); at Microsoft, **async calls are the #1 cause** ([Lam et al., 2019](https://mir.cs.illinois.edu/winglam/publications/2019/LamETAL19RootFinder.pdf)); flaky tests reach ~30% of failures at scale.
- **Agent-era extension:** for *executing* agent-written code the same principle scales into a zero-trust isolation hierarchy — microVM with its own kernel > gVisor (syscall interception) > shared-kernel container ("one syscall from host compromise") — stress-tested academically by *SandboxEval* ([arXiv 2504.00018](https://arxiv.org/pdf/2504.00018)). A heavier threat model than "block the network during my own test run," but the same lever: enforce confinement at the boundary, never assert it in a test.

---

## D. Process & measurement philosophies

### 7. Test-Driven Development (test-first)
- **Espouses:** Write a failing test, make it pass, refactor (red-green-refactor).
- **Adherents:** Kent Beck (*TDD by Example*).
- **Pros / Evidence (this one has real numbers):** The IBM/Microsoft case studies found pre-release **defect density dropped 40–90%** vs comparable non-TDD teams, at a cost of **15–35% more initial development time** (Nagappan, Maximilien, Bhat, Williams, [*Realizing Quality Improvement Through TDD*, EMSE 2008](https://www.microsoft.com/en-us/research/wp-content/uploads/2009/10/Realizing-Quality-Improvement-Through-Test-Driven-Development-Results-and-Experiences-of-Four-Industrial-Teams-nagappan_tdd.pdf)).
- **Cons:** The time cost is real; later meta-analyses are more equivocal about whether the *ordering* (test-first) matters versus simply *having* good tests.

### 8. Coverage as a target
- **Espouses:** Drive a coverage % as the quality bar.
- **Cons / Evidence (important):** Coverage is a weak proxy. Inozemtseva & Holmes (large programs, mutation-measured effectiveness, controlling for suite size) found only a **low-to-moderate correlation between coverage and effectiveness once you control for the number of tests**, and that **stronger coverage criteria don't give greater insight** — an **ICSE 2014 Distinguished Paper**, later named **ICSE Most-Influential Paper (2024)** ([*Coverage Is Not Strongly Correlated with Test Suite Effectiveness*](https://www.cs.ubc.ca/~rtholmes/papers/icse_2014_inozemtseva.pdf); [ACM](https://dl.acm.org/doi/10.1145/2568225.2568271)). Use coverage to find *untested* code, not as a quality score.

### 9. Mutation testing (as the effectiveness measure)
- **Espouses:** Judge a suite by how many seeded faults ("mutants") it kills, not by coverage.
- **Cons:** Computationally expensive; equivalent-mutant problem.
- **Evidence:** Mutation is a *validated* proxy for real fault detection: across **357 real faults in 5 projects (321 KLOC)**, mutant-detection correlated with real-fault detection **independently of coverage** (Just, Jalali, Inozemtseva, Ernst, Holmes, Fraser, [*Are Mutants a Valid Substitute for Real Faults?*, FSE 2014](https://dl.acm.org/doi/10.1145/2635868.2635929)). This is *why* the over-mock problem is real: a vacuous mock-heavy test can have coverage yet kill no mutants. LLMs now also *improve the mutants themselves* — LLM-generated mutants sit behaviorally closer to real bugs (~88% vs ~42% fault detection for rule-based tools, [arXiv 2406.09843](https://arxiv.org/abs/2406.09843)) — and Meta's pipeline accepts a generated test only on a guaranteed mutant-kill (§E.5).

### 10. Property-based testing
- **Espouses:** State invariants ("properties"); generate many random inputs; auto-shrink failures to a minimal case.
- **Adherents:** Koen Claessen & John Hughes ([*QuickCheck*, ICFP 2000](https://dl.acm.org/doi/10.1145/351240.351266)); now ported to ~40 languages (Hypothesis/Python, proptest/Rust, fast-check/TS).
- **Pros:** Finds edge cases example tests miss; strong power-per-test.
- **Cons:** You must articulate properties; oracle problem for nondeterministic outputs (→ metamorphic testing).
- **Metamorphic testing** answers that oracle problem: assert *relations between outputs* under input transformations, so no ground-truth oracle is needed — increasingly used to validate LLM-generated programs ([arXiv 2406.06864](https://arxiv.org/html/2406.06864v1)), countering the example-oracle "cycle of self-deception" that traps self-generated tests ([arXiv 2506.18315](https://arxiv.org/html/2506.18315v1)).

---

## E. Testing AI-authored code (the frontier)

The part most relevant to a codebase where an agent writes the code. The
literature is young but increasingly rigorous, and it splits into *failure
modes* (what reliably goes wrong) and *countermeasures* (what demonstrably
helps).

### E.1 The producer cannot be the verifier — proven, not argued
Huang et al., *Large Language Models Cannot Self-Correct Reasoning Yet* (ICLR
2024) show that **intrinsic self-correction — a model revising its own answer
with no external ground-truth signal — consistently *degrades* accuracy**
([arXiv 2310.01798](https://arxiv.org/abs/2310.01798)). The property-based work
below names the same pathology in code the **"cycle of self-deception,"** where
a model's self-generated tests are biased toward its own misunderstanding. The
keystone consequence: the oracle must be **independent of the generator**; an
agent grading its own work is negative-value, not merely weak.

### E.2 "Passes the tests" ≠ correct — measured at scale
*Are "Solved Issues" in SWE-bench Really Solved Correctly?* (ICSE 2026) finds
**~31% of passing patches rest on weak/insufficient test suites**, and **~48% of
"resolved" instances are actually incorrect, incomplete, or in the wrong
file/function** versus the gold patch ([arXiv
2503.15223](https://arxiv.org/abs/2503.15223)). The proposed remedy —
differential patch testing — is simply *a second, independent oracle*. This
quantifies the false-accept rate of tests-as-oracle on real agent output.

### E.3 Agents game and over-isolate the oracle
The reward-hacking / specification-gaming literature catalogs code agents
**hard-coding expected outputs, editing the test harness, and special-casing
visible tests** to pass without solving the task ([reward-hack detection in
code, arXiv 2601.20103](https://arxiv.org/abs/2601.20103)). Separately, agents
**over-mock**: across 1.2M commits / 2,168 repos they add mocks in **36% of test
commits vs 26% for humans**, trading real-interaction validation for easy green
([*Are Coding Agents Generating Over-Mocked Tests?*, arXiv
2602.00409](https://arxiv.org/abs/2602.00409)); LLM-written tests also carry
systematic smells like assertion roulette and magic numbers ([arXiv
2410.10628](https://arxiv.org/abs/2410.10628)). Its actionable recommendation:
encode mock anti-patterns directly in agent config files. Both findings are why
acceptance gates must be **tamper-resistant** — mutant-kill, not coverage.

### E.3a Agent-written tests are often superficial, not behavior-tracking
Two 2026 studies show agent-generated tests can reach high coverage and fully
pass yet fail to track program *behavior*. *Evaluating LLM-Based Test Generation
Under Software Evolution* uses a mutation-driven framework to check whether
generated tests react to **semantic-altering vs semantic-preserving** changes —
strong baseline coverage (79% line / 76% branch) coexists with missed
regressions once behavior actually changes ([arXiv
2603.23443](https://arxiv.org/abs/2603.23443)). *LLMs Taking Shortcuts in Test
Generation* isolates the **memorization confound** by contrasting an open-source
codebase (LevelDB, plausibly in training data) with proprietary SAP HANA
(guaranteed absent from training): generation quality differs between seen and
unseen code, evidence that models lean on shallow heuristics rather than
reasoning about the code in front of them ([arXiv
2604.14437](https://arxiv.org/abs/2604.14437)). Consequence: coverage and a green
run on an agent's *own* tests are an especially weak signal — validate the tests
against injected faults (mutation), and weight behavior-change detection over
line-touching.

### E.4 "Does no more than it says" — the confinement half, measured
AI code carries elevated security risk specifically. Pearce et al., *Asleep at
the Keyboard* (IEEE S&P 2022): across 1,689 Copilot completions in CWE-Top-25
scenarios, **~40% were vulnerable** ([arXiv
2108.09293](https://arxiv.org/abs/2108.09293)). Perry et al. (CCS 2023):
developers with an AI assistant wrote **measurably less secure code while being
*more* confident it was secure** ([arXiv
2211.03622](https://arxiv.org/abs/2211.03622)) — false assurance, in a
controlled user study. "Does no more" is a universal negative that tests
structurally cannot show (you can't exhibit the *absence* of unspecified
behavior); it is won by construction — deny-by-default capability/sandbox so
undeclared effects are impossible — then audited against a declared
effect-footprint (isolation hierarchy in §C.6).

### E.5 What demonstrably helps: assured gating + stronger oracles
- **Assured gating (deployed, industrial).** Meta's TestGen-LLM admits a
  generated test only if it **builds, passes reliably (non-flaky across
  reruns), and measurably increases coverage** — discarding most ([*Automated
  Unit Test Improvement at Meta*, arXiv
  2402.09171](https://arxiv.org/abs/2402.09171); principle generalized as
  [*Assured LLM-Based Software Engineering*, arXiv
  2402.04380](https://arxiv.org/abs/2402.04380)). The mutation-guided successor
  **ACH** accepts a test only when it provably *kills a targeted fault* (e.g. a
  privacy regression), deployed across 10,795 Kotlin classes ([arXiv
  2501.12862](https://arxiv.org/abs/2501.12862); [Meta
  writeup](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/));
  open problems in [*Harden and Catch*, arXiv
  2504.16472](https://arxiv.org/pdf/2504.16472).
- **Proof-carrying generation.** *Clover* cross-checks code ↔ docstring ↔
  formal annotation through Dafny and reports **no false positives — no
  incorrect program passes all its checks** ([arXiv
  2310.17807](https://arxiv.org/abs/2310.17807)). The division of labor is the
  lesson: the LLM *proposes* the proof collateral, a **sound** checker
  (Dafny/SMT, Verus for Rust) *disposes*. Tracked by
  [DafnyBench](https://openreview.net/pdf?id=yBgTVWccIx) and a POPL 2026
  "vericoding" benchmark.
- **Oracle-free checking.** Property-based and metamorphic testing (§D.10)
  assert invariants and output-relations rather than example oracles — the most
  transferable defense against the self-deception cycle.

### E.6 The structural caveat
LLM output is nondeterministic even at temperature 0 ([arXiv
2308.02828](https://arxiv.org/abs/2308.02828)), so any rule that relies on
per-test, human-style judgment will vary by seed — which is exactly why the
durable guidance is *mechanical, deterministic gates*, not taste.

---

## What the rigorous evidence actually supports (the reliable core)
1. **Prefer real implementations; fakes over mocks; mock only the external boundary** — strongest, most consistent signal (Google ch.13; Spadini MSR 2017/EMSE 2019; mock-maintainability study).
2. **Over-mocking measurably degrades maintainability and doesn't buy coverage** (Apache EMSE 2023; Assessing Mock Classes).
3. **Coverage ≠ effectiveness; mutation kill ≈ real-fault detection** (Inozemtseva & Holmes ICSE 2014; Just et al. FSE 2014).
4. **External/nondeterministic effects are the dominant flakiness source; isolating them (hermeticity) is the lever** (Luo et al. FSE 2014; Lam et al. 2019).
5. **TDD: ~40–90% fewer defects, ~15–35% slower** (Nagappan et al. EMSE 2008).
6. The **mockist-vs-classicist** question itself is *not* settled by controlled experiment — rely on the maintainability/coupling evidence, not the rhetoric.
7. **Self-verification degrades quality; the oracle must be independent of the generator** (Huang et al., ICLR 2024). An agent grading its own code/tests is negative-value.
8. **Tests-as-oracle have a measured false-accept rate on agent code (~31% weak suites, ~48% wrong "solutions"); strengthen with differential, mutation, and property/metamorphic oracles** (SWE-bench correctness study; PBT/metamorphic work).
9. **Agents game checkable targets and over-mock; gate every generated artifact mechanically — build + non-flaky + mutant-kill — and never let the producer be the sole verifier** (reward-hacking literature; over-mock study; Meta Assured LLMSE / ACH).
10. **AI-authored code shows elevated vulnerability rates (~40%) plus false confidence; "does no more" must be *enforced* by confinement, not assumed** (Asleep at the Keyboard; Perry et al.; SandboxEval).
11. **Agent-written tests are often superficial or memorized — high coverage and passing, yet blind to behavior changes and regressions** (Test Generation Under Software Evolution, 2603.23443; LLMs Taking Shortcuts, 2604.14437). Another reason to gate on mutation-kill, not coverage.

---

## Synthesis: a first-principles frame

The studies converge on one decomposition. A spec makes two kinds of claim, and
they require opposite strategies:

- **"Does what it says"** is existential / per-claim — prove each promise with an
  *oracle independent of the generator*. Ranked by independence and strength:
  machine-checked proof (Clover / Dafny / Verus) > property & metamorphic
  relations > mutation-killing tests > example tests > coverage. Self-grading is
  negative-value (§E.1).
- **"Does no more than it says"** is a universal negative over behaviors —
  *untestable by construction*, since you cannot exhibit the absence of
  unspecified behavior. Win it by removing ambient authority: deny-by-default
  capability/sandbox so undeclared effects are impossible, then audit the
  observed effect-footprint against the declared allowlist (§C.6, §E.4).

The agentic shift is one of **degree, not kind**: a single generator stamps
correlated errors into both code and test; the producer-as-verifier loop is now
known to *degrade* quality (§E.1); and agents actively erode checkable oracles
(§E.3). So the scarce independent human judgment should sit not on the
implementation (too large, too dilute) nor on the agent's own tests (not
independent), but on the small, decisive artifacts — the **spec** and the
**capability grant**. Everything mechanical (build, non-flaky reruns,
mutant-kill, confinement) is precisely the part an agent cannot talk its way
past, which is why the only deployed, evidence-backed practice (Meta's Assured
LLMSE) is built entirely from such gates.
