# Summary

- **Title:** Realizing quality improvement through test driven development: results and experiences of four industrial teams
- **Authors:** Nachiappan Nagappan, E. Michael Maximilien, Thirumalesh Bhat, Laurie Williams
- **URL:** https://www.microsoft.com/en-us/research/wp-content/uploads/2009/10/Realizing-Quality-Improvement-Through-Test-Driven-Development-Results-and-Experiences-of-Four-Industrial-Teams-nagappan_tdd.pdf
- **Date:** Published online 27 February 2008
- **Venue:** Empirical Software Engineering (2008) 13:289–302, DOI 10.1007/s10664-008-9062-z (Springer)
- **Source type:** (peer-reviewed / industry case-study; full paper PDF captured)

## What the source claims

Four industrial case studies (three teams at Microsoft, one at IBM) that adopted TDD or a
TDD-inspired practice. Central claim: pre-release defect density of the four TDD products
dropped 40–90% relative to comparable non-TDD projects, at the cost of a subjectively
estimated 15–35% increase in initial development time.

Verbatim quotes:

> "The results of the case studies indicate that the pre-release defect density of the four
> products decreased between 40% and 90% relative to similar projects that did not use the
> TDD practice. Subjectively, the teams experienced a 15–35% increase in initial development
> time after adopting TDD."

> "With this practice, a software engineer cycles minute-by-minute between writing failing
> unit tests and writing implementation code to pass those tests."

> "All the teams demonstrated a significant drop in defect density: 40% for the IBM team;
> 60–90% for the Microsoft teams."

> "From an efficacy perspective this increase in development time is offset by the by the
> reduced maintenance costs due to the improvement in quality ... an observation that was
> backed up the product teams at Microsoft and IBM."

> "TDD seems to be applicable in various domains and can significantly reduce the defect
> density of developed software without significant productivity reduction of the development
> team."

On the Microsoft "hybrid-TDD" practice:

> "By hybrid we mean that these projects as with almost all projects at Microsoft had
> detailed requirements documents written ... This explains our reason to call this a
> hybrid-TDD approach, as agile teams typically do not have design review meetings."

## Method / evidence type

- **Four in vivo industrial case studies**: IBM (device drivers; new TDD platform compared to
  the 7th release of a legacy non-TDD platform) and Microsoft Windows, MSN, and Visual Studio
  (Developer Division) teams, each compared to a similar non-TDD project under the same
  higher-level manager.
- Defect data mined from IBM's and Microsoft's bug databases; quality measured as **defect
  density** (defects/KLOC) post-integration.
- Development-time increase obtained from **subjective management estimates** (not measured).
- Projects "analyzed post hoc"; developers did not know during development they would be
  studied.
- Authors stress these are case studies ("research in the typical"), not controlled
  experiments, and a "family of case studies is likely not to yield statistically
  significant results."

## Numbers recorded

Product factors (Table 2):

| Metric | IBM Drivers | MS Windows | MS MSN | MS VS |
|---|---|---|---|---|
| Source KLOC | 41.0 | 6.0 | 26.0 | 155.2 |
| Test KLOC | 28.5 | 4.0 | 23.2 | 60.3 |
| Test/Source KLOC | 0.70 | 0.66 | 0.89 | 0.39 |
| Block coverage from unit tests (%) | 95 | 79 | 88 | 62 |
| Development time (man-months) | 119 | 24 | 46 | 20 |

Outcome measures (Table 3), defect density of TDD team relative to a comparable non-TDD team
(W, X, Y, Z denote each non-TDD baseline):

| | IBM Drivers | MS Windows | MS MSN | MS VS |
|---|---|---|---|---|
| Non-TDD comparable team | W | X | Y | Z |
| TDD team | 0.61W | 0.38X | 0.24Y | 0.09Z |
| Increase in coding time due to TDD (mgmt estimate) | 15–20% | 25–35% | 15% | 25–20% |

(0.61W ≈ 39% reduction, 0.38X ≈ 62%, 0.24Y ≈ 76%, 0.09Z ≈ 91% — reported as "40%" for IBM
and "60–90%" for Microsoft.) "The increase in development time ranges from 15% to 35%."

Related-work figures cited: George & Williams structured experiment with 24 professional
programmers — TDD programmers "passed 18% more functional black-box test cases" but "took 16%
more time." Müller & Tichy university study (11 students): "87% of the students stated that
the execution of the test cases strengthened their confidence in their code." Humphrey 1989
cited: maintenance fixes/small changes "may be nearly 40 times more error prone than new
development."

Team factors (Table 1): team sizes 9 (IBM), 6 (Windows), 5–8 (MSN), 7 (VS); IBM team
distributed across Raleigh, NC and Guadalajara, Mexico; languages Java, C/C++, C++/C#, C#.
IBM target: "cover at least 80% of the developed classes by automated unit testing."

## Scope, limitations, and gaps

Stated by the authors (Section 7, Threats to Validity):

- **Novelty / motivation bias:** TDD developers might have been more motivated trying a new
  process; alleviated because teams did not know they would be studied.
- **Comparability:** "there can never be an accurate equal comparison between two projects
  except in a controlled case study"; IBM compares a new TDD project against an enhancement to
  a legacy non-TDD system, which could bias defect density in either direction (legacy code
  already field-tested vs. legacy code carrying existing problems).
- **Statistical power:** "a family of case studies is likely not to yield statistically
  significant results."
- **Generalizability:** "we cannot assume a priori that the results of a study generalize
  beyond the specific environment in which it was conducted."
- Development-time increase is a subjective management estimate, not measured; maintenance-cost
  offset is asserted, not quantified.
- Note: a later release of the IBM product (team grown 50%) saw defect density rise again
  "temporally" when some members skipped running the unit tests.

## Capture status

`transcript.md` is the full paper PDF (`transport: curl`, `capture_status: ok`). It contains
the abstract, full body (Sections 1–8), Tables 1–3, references, and author bios. Figures 1–2
are diagrams that did not extract as data, but they are conceptual (context factors, TDD
methodology overview) and carry no numeric results beyond the tables transcribed above.
