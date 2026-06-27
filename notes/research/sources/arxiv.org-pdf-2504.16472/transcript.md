---
url: https://arxiv.org/pdf/2504.16472
title: Harden and Catch for Just-in-Time Assured LLM-Based Software
fetched: 2026-06-27
raw: raw.pdf
transport: curl
capture_status: ok
---

Harden and Catch for Just-in-Time Assured LLM-Based Software
                                                      Testing: Open Research Challenges
                                                             Mark Harman                                                 Peter O’Hearn                              Shubho Sengupta
                                                 Product Compliance and Privacy                           Fundamental AI Research team, Meta                   Independent Software and AI
                                                      team, Meta Platforms                                            Platforms                                        Consultant
                                                           London, UK                                                London, UK                                           USA
                                                    University College London                                 University College London
                                                    London, United Kingdom                                     London, United Kingdom
arXiv:2504.16472v2 [cs.SE] 14 May 2025




                                         ABSTRACT                                                                                    1   INTRODUCTION
                                         Despite decades of research and practice in automated software                              For the past decade, at Meta, we (the authors) have been developing
                                         testing, several fundamental concepts remain ill-defined and under-                         automated static and dynamic analysis and testing technologies.
                                         explored, yet offer enormous potential real-world impact. We show                           We have also been involved in the development of AI technologies,
                                         that these concepts raise exciting new challenges in the context of                         as a support to such analyses, and also as an independent technol-
                                         Large Language Models for software test generation. More specifi-                           ogy in its own right. As well as our recent industrial experience,
                                         cally, we formally define and investigate the properties of hardening                       we have been active in the Software Engineering, Programming
                                         and catching tests. A hardening test is one that seeks to protect                           Languages, and Artificial Intelligence research communities over
                                         against future regressions, while a catching test is one that catches                       several decades. Our observation is that, despite much research and
                                         such a regression or a fault in new functionality introduced by a                           development, some of the most fundamental concepts that lie at
                                         code change. Hardening tests can be generated at any time and                               the intersection between scientific research and practical software
                                         may become catching tests when a future regression is caught. We                            testing have yet to be adequately analysed, formalised, or defined,
                                         also define and motivate the Catching ‘Just-in-Time’ (JiTTest) Chal-                        let alone fully tackled.
                                         lenge, in which tests are generated ‘just-in-time’ to catch new faults                         Research and practice are unequivocally and rapidly moving
                                         before they land into production. We show that any solution to                              towards automated software test design, increasingly relying on
                                         Catching JiTTest generation can also be repurposed to catch latent                          language models to generate the test cases [26]. This makes it more
                                         faults in legacy code. We enumerate possible outcomes for hard-                             important than ever to have crystal clear definitions of the most
                                         ening and catching tests and JiTTests, and discuss open research                            pressing problems facing practising software engineers, when it
                                         problems, deployment options, and initial results from our work on                          comes to automated test generation [11]. Automated test generation
                                         automated LLM-based hardening at Meta. This paper1 was written                              is not only important for testing code generated, both by humans
                                         to accompany the keynote by the authors at the ACM International                            and models, it may also prove useful for training and assessing the
                                         Conference on the Foundations of Software Engineering (FSE) 2025.                           models themselves [50].
                                                                                                                                        In this paper, we attempt to provide simple, intuitive, and pre-
                                         CCS CONCEPTS                                                                                cise formulations of the underlying concepts on which automated
                                         • Software and its engineering → Software testing and debug-                                software test design rests. We first formalise the way in which gen-
                                         ging.                                                                                       erated tests harden against future regressions. This regression test
                                                                                                                                     generation has been the primary focus of both academic research
                                         KEYWORDS                                                                                    and practice in automated test generation for the past few decades,
                                                                                                                                     with notable successes [6, 17, 36, 84]. However, as we show, it leads
                                         Software Engineering, Programming Languages, Large Language
                                                                                                                                     us into a Regression Only Trap, in which tests must pass on the
                                         Models (LLMs).
                                                                                                                                     current revision, and therefore cannot catch new bugs introduced
                                         ACM Reference Format:                                                                       by that revision, nor long-standing legacy bugs.
                                         Mark Harman, Peter O’Hearn, and Shubho Sengupta. 2025. Harden and                              These regression tests are ‘hardening’ tests according to our
                                         Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research
                                                                                                                                     terminology, because they seek to detect bugs in future changes
                                         Challenges. In 33rd ACM International Conference on the Foundations of Soft-
                                         ware Engineering (FSE Companion ’25), June 23–28, 2025, Trondheim, Norway.
                                                                                                                                     rather than in the current system; they ‘harden’ against future
                                         ACM, New York, NY, USA, 17 pages. https://doi.org/10.1145/3696630.3734199                   regressions. Research and practice has tended to focus on regression
                                                                                                                                     testing due to the Oracle Problem [13]. Regression testing offers the
                                         1 Author order is alphabetical. The corresponding author is Mark Harman.
                                                                                                                                     automated test tool designer a readily deployable solution to the
                                         Permission to make digital or hard copies of all or part of this work for personal or
                                                                                                                                     Oracle Problem: the Regression Oracle [7]. The Regression Oracle
                                         classroom use is granted without fee provided that copies are not made or distributed       dictates that the behaviour of the previous version of the system
                                         for profit or commercial advantage and that copies bear this notice and the full citation   should be preservered by any change, thereby ensuring that there
                                         on the first page. Copyrights for third-party components of this work must be honored.
                                         For all other uses, contact the owner/author(s).                                            are no regressions.
                                         FSE Companion ’25, June 23–28, 2025, Trondheim, Norway
                                         © 2025 Copyright held by the owner/author(s).
                                         ACM ISBN 979-8-4007-1276-0/2025/06
                                         https://doi.org/10.1145/3696630.3734199
FSE Companion ’25, June 23–28, 2025, Trondheim, Norway                                                                   Harman, O’Hearn, Sengupta


    In Section 2 we briefly review our previous work on automated          2     AUTOMATED TESTING AT META
test generation at Meta, leading up to our current work on LLM-            In this section, we briefly review our previous and current work, at
based automated unit test generation for hardening. The primary            Meta, on automated test generation. The section is a brief review of
contributions of this work lie in the way we use LLMs and mutation.        work published in more detail elsewhere. The aim of the section is
    Nevertheless, by remaining stuck in the Regression Only Trap,          to make the paper self-contained and to motivate the need for the
we (and the wider the community), have overlooked an impor-                more detailed analysis of hardening and catching tests, introduced
tant untackled problem: the Catching Just-in-Time test (Catching           in Section 3.
JiTTest) problem. A JiTTest is generated when a pull request is
submitted. We also formalise and analyse JiTTests. By seeking to
generate a catching JiTTest, we aim to catch bugs specifically in          2.1    Background
that pull request. A JiTTest is not concerned with some possibly           At Meta, we have previously deployed end-to-end test generation
distant future in which some regression may (or may not) occur,            as well as unit-level test generation. Our end-to-end testing tools in-
but the immediate future in which a pull request is about to land          clude Sapienz [60], deployed since 2017 [6] covering client-side test-
into production.                                                           ing on all platforms, and Fausta, deployed since 2021 [61], covering
    Generating Catching JiTTests allows us to escape the Regression        server-side testing for WhatsApp. The initial Sapienz deployment in
Only Trap (ROT), but it represents a more challenging problem              2017 used the implicit oracle [13], which has remained widespread
than generating hardening regression tests, although it subsumes           throughout the company since 2017. More recently, we enhanced
that problem. It is also much more impactful. A JiTTest can catch          the Sapienz oracle to extend beyond the implicit oracle [79], and to
bugs in new functionality as well as regressions, and it stops them        cater for the rich diversity of user states needed to cover the state
literally just-in-time. That is, just before the pull request lands into   space [4].
production. Because they do not merely target regressions, JiTTests           These end-to-end testing systems can be thought of as
can also easily be repurposed to find latent bugs in the existing code     simulation-based testing approaches [1], for which we have used
base, as we show in Section 6.5. We believe the Catching JiTTest           Metamorphic Testing to test the simulation itself [2]. In traditional
Challenge to be the most challenging and impactful problem in              end-to-end testing, a test case simulates the end-to-end behaviour
software testing. Although there has been initial discussion of just-      of the platform with respect to a single user. We have also deployed
in-time testing approaches [14, 28, 65, 81], much more remains to          simulation-based test generation techniques that simulate the end-
be done.                                                                   to-end behaviour of an entire interacting community of users. We
    Section 3, introduces our formalisation of hardening and catch-        call this approach ‘social testing’.
ing. Our formalism allows us to precisely determine when a harden-            Social testing is not only applicable to any platform in which
ing test subsequently catches a bug. It also allows us to distinguish      users interact with each other, it is also necessary so that these
between tests that catch regressions and those that catch bugs in          interactions are fully tested [3, 79]. As we have shown in our previ-
new functionality. We use our formalism to precisely define the            ous work, social testing goes beyond traditional end-to-end testing,
Catching JiTTest Challenge. Our definitions apply to testing in            because it takes into account user ‘personas’, and the fact that the
general, whether tests are generated by machine or by human, and           test oracle for one user differs (and may even contradict) the test
irrespective of the generation technique used. Nevertheless, the           oracle for another user.
motivation for our work is our recent development and deployment              These dynamic analyses are complemented by static analy-
[5, 29] of LLM-based test generation (using an Assured LLMSE               ses [23], such as Infer [18] and Zoncolan.
approach [8]).                                                                In addition to end-to-end test generation, we have also more
    The paper also includes a case-based analysis of possible deploy-      recently developed unit-level test generation techniques using lan-
ments for hardening and catching tests, considering the balance of         guage models [5] and observations [9]. These unit test generation
risk and reward inherent in their deployment. This analysis reveals        techniques focus on covering uncovered code.
new insights into deployment options.                                         To go beyond merely achieving additional structural code cov-
    The primary contributions of this paper are as follows:                erage, we also developed a mutation-guided approach [29]. The
                                                                           mutation-guided approach ensures that the generated tests find
                                                                           faults that no other test can find. This more recent mutation-guided
   (1) Introduction of the Catching JiTTest Challenge: Al-                 approach generates hardening tests (according to Definition 6), be-
       though challenging, we argue that the Catching JiTTest Chal-        cause the tests are guaranteed to find at least one currently uncaught
       lenge is the most highly impactful currently open challenge         potential future fault.
       in automated test generation. We also show how advances
       in LLMs open the door to previously unavailable solutions,
       making the time ripe to tackle Catching JiTTest Challenge.
                                                                           2.2    Assured Large Language Model Based
   (2) Precise Formal Foundations: A formalisation of hardening                   Software Testing (Assured LLMST)
       and catching tests, and the distinction between regression          Our approach to the generation of hardening tests is grounded in
       catching and functionality catching tests.                          the principles of Assured LLM-based Software Testing (Assured
   (3) Detailed Practical Deployment Analysis: A case-by-case              LLMST). Assured LLMST is simply Assured LLM-Based Software
       analysis of deployment options, illustrating how the formal-        Engineering (Assured LLMSE) applied to the field of Software Test-
       ism informs practical deployment options.                           ing.
Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges            FSE Companion ’25, June 23–28, 2025, Trondheim, Norway


    Assured LLMSE [8] is a generate-and-test approach to LLM-                        between online and offline LLMSE) rests on the definition of ‘real
based software generation, inspired by the Genetic Improve-                          time’, which has its own notion of timeliness [24, 55]. More for-
ment [69] approach to Search Based Software Engineering [37, 38,                     mally, the LLMSE definition [8] of real time is repeated below as
73], also widely used in Software Testing [36, 63]. Assured LLMST                    Definition 1.
is for Assured LLMSE, what Search Based Software Testing (SBST)
is for Search Based Software Engineering (SBSE); it focusses on that                     Definition 1 (Real Time, taken from ref [8]). A result is
subset of Software Engineering that is concerned with Software                       provided in real time for a consumer 𝑐 and application 𝑎, if and only
Testing.                                                                             if 𝑐 cannot perform 𝑎 any better when the time spent awaiting the
    Assured LLMSE addresses the two fundamental questions: How                       LLM response reduces.
can we provide verifiable assurances that the LLM-generated code                         This is a ‘utility’-based definition of real time. It focusses on the
   (1) does not regress the properties of the original code ?                        code being provided in sufficient time to be acted upon. Any JiTTest
   (2) improves the original in a verifiable and measurable way ?                    generation approach that contributes tests generated on the fly for
                                                                                     a pull request merely has to generate these tests before the first
2.3     Assured LLMST is Easier than Assured                                         human reviewer considers the pull request.
        LLMSE                                                                            It may take a human reviewer hours or even days to provide that
Fortunately Assured LLMST is considerably less challenging, ac-                      first review. According to a recent study [42], the median response
cording to both these constraints, then LLMSE.                                       time for Git projects was estimated at 1.75 hours. This estimate
Does not Regress: In the context of software testing, not regress-                   includes (instant response) static analysis bots (such as lint tools),
ing the properties of the original code means not regressing the                     which tends to make it an underestimate of the time available to the
properties of the original test suite. For a test suite, the typical                 first human response. Even with this estimate, any test generated
properties of interest lie in its ability to achieve a certain level                 within 1.75 hours would thus be ‘in time’ for the review process
of coverage and to catch a certain class of faults. When we add                      with a probability of at least 0.5.
new test cases to an existing test suite (but do not remove any),                        In fact, a more realistic estimate of the typical industry standard
we cannot, by definition, regress the test suite’s ability to achieve                time for the first human response to a pull request is closer to 8
coverage, nor its ability to reveal faults. Provided we treat each                   hours [62]. That is, there is an aspiration to respond within eight
test case as a unique discrete unit (on which existing tests have no                 hours, and this is also therefore a realistic bound for ‘real time’ test
dependency), then adding additional test cases can only improve                      generation (and thus for a JiTTest). As can be seen, the constraints
the overall test suite. This is in stark contrast to the situation in                for a test to be considered a JiTTest are not as tight as they might
which we use language models to generate code other than tests.                      initially appear. This is important because eight hours affords a
When generating code in general, it is easy to accidentally regress                  great deal of computational time that our tools and infrastructure
existing functionality.                                                              can use to evaluate the assurances that can be offered to the engineer
Improves the Original: For the application to software testing,                      as part of the review process.
improving the original in a verifiable and measurable way is also
simple: We can measure the additional coverage and/or can give                       2.5     Initial Results on LLMST
an assurance that the new test finds a fault that is uncaught by any                 At Meta, we recently developed and deployed a mutation-guided
existing test. By contrast, the goal of improving code, other than                   test generation system called ACH [29]. ACH ensures that the
test code, in a verifiable and measurable way can be challenging.                    generated tests are genuinely hardening; they can catch a potential
For example, attempting to improve code execution time raises the                    future bug. This should be contrasted with test generation that aims
challenges of measuring execution time reliably, over a distribution                 to improve structural code coverage but may not necessarily prove
of potential inputs.                                                                 to be fault-revealing. Furthermore, since our deployment places test
                                                                                     generation in a scenario in which the mutant is guaranteed to be
2.4     Online Assured LLMST                                                         killed, we also sidestep the familiar equivalent mutant problem [41,
As with all machine learning [46], LLMSE can be either ‘online’                      51, 58, 68, 80].
or ‘offline’ [8]. The extra time available to offline LLMSE typically                   The overall approach allows the ACH tool to give the following
allows the Assured LLMSE application greater opportunities to                        six assurances, as part of its overall Assured LLMSE approach.
provide assurances.                                                                      (1) Buildable: The proposed new tests build, so are free from
   One might naturally think of an ‘online’ approach as one that                             syntax errors and missing dependencies;
responds in real time. The archetype of online LLMSE is code com-                        (2) Valid Regression Tests: The tests pass and do so consis-
pletion. Code completion technologies, such as CoPilot [85] and                              tently, so they are non-flaky [39] regression tests;
CodeCompose [66], provide the generated code directly in the editor                      (3) Hardening: The new tests catch faults that no existing test
as the software engineer types text. This is clearly online because                          can catch. Our mutation-guided tests are ‘hardening’ accord-
the response latency is the same as the duration of a keystroke.                             ing to Definition 6. Furthermore, when an engineer accepts
   When we formalise the notion of online, it becomes clear that                             and lands such a generated test into production, the test
there is, perhaps surprisingly, far more time available for some                             becomes a Strong Hardening Test according to Definition 7.
generated test deployment options than there is for code comple-                         (4) Additional Coverage: The tool automatically reports any
tion. The definition of ‘online’ (for the purposes of distinguishing                         additional coverage achieved.
FSE Companion ’25, June 23–28, 2025, Trondheim, Norway                                                                             Harman, O’Hearn, Sengupta


    (5) Relevant: Many of the new tests are closely coupled to the           (1) A tree repository with a function Parent(𝑅) for a revision (aka
        issue of concern;                                                        pull request) 𝑅.
    (6) Fashion Following: Most of the new tests respect the cod-            (2) A set Test Cases, 𝑇
        ing style used by the existing tests available for the class         (3) Predicates Builds𝑅 (𝑡), Passes𝑅 (𝑡), Fails𝑅 (𝑡), for a revision 𝑅
        under test; they are ‘fashion followers’ [5].                            and test case 𝑡.

   The first four of these six assurances are assurances in the strict        The intuition behind these relations is as follows. When a test
meaning of the word: they are verifiable (and, therefore, falsifiable)    case, 𝑡 builds on revision 𝑅 this is denoted Builds𝑅 (𝑡). A test case
guarantees that leave no room for doubt. The final two stretch the        that builds can be executed. When executed, a test yields a signal
definition of ‘assurance’ a little bit; they are non-boolean aspira-      that includes at least2 two possible outcomes, pass and fail. When
tional attempts to provide improvement, in the sense of genetic           the outcome of executing test 𝑡 on revision 𝑅 is pass, this is denoted
improvement guarantees [69]. That is, we hope that the test will          Passes𝑅 (𝑡). When the outcome of executing test 𝑡 on revision 𝑅 is
‘fashion follow’, and therefore be more intuitive to the human reader,    fail, this is denoted Fails𝑅 (𝑡). A Test suite 𝑇 is simply a set of test
but this is something that can be somewhat subjective, rather like        cases. Executing a test suite simply means executing each of the
genetic improvement and restructuring [40, 71] or other aesthetic         tests in the suite.
software improvements [72, 76]. We also hope the tests will be                We make the simplifying assumption that tests can be executed
relevant, but this is also a somewhat subjective judgment.                independently. That is, the execution of one test from a suite does
   We found that tests designed to be hardening (by killing mutants)      not influence the outcome of some other test from the suite. All of
that were deemed to be irrelevant by the engineer reviewing them,         our definitions and concepts can be adapted to relax this indepen-
were more likely to be accepted when they improved coverage.              dence assumption, but it makes the treatment more complex.
Such tests proved acceptable even when they did not have another              At first sight, it might seem that all tests should/would be build-
contribution [5, 29]. Of course, coverage improvement is not a            able, but this is incorrect. When a new revision is created, some of
necessary criterion for a test to be deemed acceptable because the        the existing tests may become invalid (unbuildable). For example,
test may reveal a fault without covering any new lines of code.           if the name of a method is changed, then any test that relies on the
   Therefore, we report additional coverage as a spinoff benefit; it is   method name will no longer build. Therefore, for each revision, not
a measurable assurance we can track, but do not target. In managing       all tests will necessarily be buildable.
software engineering deployment, it is important to distinguish               We are concerned with test timeliness, because we want to define
between metrics that merit tracking from those that should become         just-in-time test generation. We therefore define a timely test, but
goals and Key Performance Indicators.                                     do not require that it be executable, merely that it be available:
   More details about our work on LLM-based test extension can be            Definition 3 (Timely Test). A timely test for the revision 𝑅, is
found in our FSE 2024 paper [5], while the more recent approach to        available for attempted execution on 𝑅. That is, the test exists on or
incorporate mutation testing to guide the generation of tests toward      before the time at which the revision 𝑅 is first submitted as a pull
those that harden with respect to specific faults (using mutation)        request for review.
can be found in our FSE 2025 paper [29].
                                                                             We want tests to be available when needed, but we also want
                                                                          to define a specific kind of test that is literally ‘just-in-time’ for a
3    FORMALISING HARDENING AND                                            revision 𝑅:
     REGRESSION AND FUNCTIONALITY
                                                                             Definition 4 (Just-In-Time Test (JiTTest)). A Just-in-Time
     CATCHING TESTS                                                       test, or ‘JiTTest’, for revision 𝑅, is a timely test for 𝑅 that is not a
In this section we formalise the notions of hardening and catching        timely test for Parent(𝑅).
tests using these to subsequently distinguish regression catching
and functionality catching tests. The formalism allows us to make            A JiTTest is a kind of on-the-fly [28, 81] or just-in-time test [14,
these distinctions more precise. In Section 5 we use it to analyse,       65], in which we seek to construct a test as soon as the revision
case by case, the possible signals we may obtain from tests, the pos-     appears as a pull request. An engineer could write a JiTTest. Indeed,
sible underlying causes, and consequent courses of action available       many engineers do write unit tests just-in-time, because they create
when deploying hardening and catching test generation.                    them to accompany their pull requests.
   Software engineers typically deploy their code into a continuous          In this paper, we are particularly interested in generated JiTTests,
integration pipeline, consisting of a sequence of ‘revisions’ (aka        because this is an application of test generation that has been largely
‘versions’ or ‘pull requests’). We purposely do not over-constrain        ignored in the research literature. The JiTTest may harden the
the concept of a version. This could be a version that has been           pull request against future regressions (Hardening JiTTest) or may
released, or one that is purely internal. The version may reside on       catch new bugs in it (Catching JiTTest). As we shall see, Catching
a ‘forked’ branch or may reside on the ‘trunk’ of the commit tree.        JiTTest generation represents the most challenging, but also most
For the purposes of defining Just-in-Time software test generation,       impactful application area for automated test generation. We are
these details are irrelevant; we simply need a way to refer to the        2 There are often other possibilities such as timeouts, which may signify an engineering

parent of a given code version.                                           concern about incorrect test infrastructure operation. We do not want to constrain
                                                                          our definitions to include these signals, since they are engineering details that are
                                                                          irrelevant to understanding the fundamental signal of a correctly operating hardening
    Definition 2 (Fundamentals). Defintion: We assume                     or catching test.
Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges            FSE Companion ’25, June 23–28, 2025, Trondheim, Norway


also interested in timely hardening test generation. Although this is                   These empirical observations are also born out by our definitions.
less challenging that Catching JiTTest generation, it is also useful,                A test can increase coverage, yet be guaranteed, by construction,
and represents the current deployment mode for most automated                        to never reveal any bugs. For example, consider a test that exe-
test generation, both in the research and practitioner communities.                  cutes additional lines of code yet relies on an oracle (assertion)
   We do not attempt to formally define ‘generated test’ for revision                that is semantically equivalent to Assert(True). This is a rather
𝑅, but our intuition is that a generated test is simply any test that                extreme case of the well-known coverage dilemma, but it illustrates
is generated by machine, without human supervision.                                  the fundamental underlying problem. To address this, we define
                                                                                     a ‘hardening’ test case; one that hardens the code against future
                                                                                     changes by catching at least one bug or type of bug in a well-defined
3.1     Coverage                                                                     way.
We want to be able to distinguish useful tests from useless (e.g.,
trivial) tests. For example, the trivial test Assert(True) is guaran-                3.2     Oracles
teed to be both timely and buildable for any revision, according to                  In order to define a hardening test case, we first need to introduce
our definitions, but clearly offers no value.                                        the concept of an oracle [13]. Strictly speaking, we do not require a
    Coverage is currently the most widely used criterion for test                    full oracle, but merely a partial oracle that is able to determine the
generation in both academic research and industrial deployment.                      outcome for each test case on each revision. This is an important
For example, there are automated test data generation systems                        distinction, because we do not necessarily require a full specification
based on Search Based Software Testing (SBST) [36, 63] such as                       of the system, but merely an answer to the question: ‘should test 𝑡
Austin [56] for C and EvoSuite [31] for Java, and test generation                    have passed or failed on revision 𝑅?’
systems based on concolic execution [17, 32, 75] such as Klee [16]
that are publicly available and have been used both in research                        Definition 5 (Oracle). Given a test 𝑡 and a revision 𝑅, Oracle
work and in practical deployment. These technologies focus on                        Oracle𝑅 (𝑡) gives the expected result for test 𝑡 in 𝑅.
coverage achieved, seeking to increase coverage by making it a                            𝑡 is a true positive for 𝑅 if Fails𝑅 (𝑡) ∧ Oracle𝑅 (𝑡) = fail.
search objective, in the case of SBSE deployment, or making it the                       𝑡 is a false positive for 𝑅 if Fails𝑅 (𝑡) ∧ Oracle𝑅 (𝑡) = pass.
focus of a mixture of symbolic and concrete execution in the case                       𝑡 is a true negative for 𝑅 if Passes𝑅 (𝑡) ∧ Oracle𝑅 (𝑡) = pass.
of concolic execution systems.                                                          𝑡 is a false negative for 𝑅 if Passes𝑅 (𝑡) ∧ Oracle𝑅 (𝑡) = fail.
    Automated end-to-end test generation systems such as
Sapienz [6, 60] and other simulation-based testing technologies,                        We leave the question of how one determines the oracle delib-
such as Meta’s WW [3] and Fausta [61], and Google’s RecSim [48]                      erately unspecified. In much of the literature on oracles [13] there
also increase coverage, but as a side benefit of seeking to catch bugs.              is an implicit assumption that there exists a single ground-truth
These end-to-end systems target the detection of bugs rather than                    oracle for a software system, albeit one which may be unknown
the elevation of coverage. They tend to rely on the so-called implicit               or only partially specified, but this is an unreasonable assumption.
oracle [13] or the regression oracle [7], so they cannot target the                  This was pointed out as early as 1979 with the publication of a
revelation of faults in new functionality.                                           seminal paper [21]. That paper was deemed, by some, to be contro-
    Automated fuzzing techniques [59], such as AFL [84] have also                    versial [22] at the time. Nevertheless, it has become increasingly
been proven to be successful in industrial deployment, due to their                  apposite as the nature of Software Engineering has evolved over the
conceptual simplicity, applicability and ability to catch security                   intervening years [44, 45, 67]. Specifically, Software Engineering is
flaws. These technologies also tend to focus on increasing the cov-                  a partly social process, so one engineer’s (and/or one user’s) oracle
erage of the system under test and use the implicit oracle, tradi-                   may differ from another.
tionally focusing on features that, in any and all applications, could                  We do not wish to enter into this detail here so we assume that,
lead to a security flaw.                                                             given a revision and a test, there is agreement on the oracle for the
    The implicit oracle captures that which no software system                       expected behaviour of the test on that revision. For example, for
should ever do (such as crashing and running out of memory),                         practical purposes, we may consider that the decision is confined to
rather than the properties that we expect for a specific system (such                the author of the pull request, since this is the person (or machine)
as its functional and other behavioural properties). This means that                 who, for all practical purposes, will play the role of deciding whether
implicit oracle tests can only target a very restricted, albeit highly               a test signal is acceptable.
egregious, class of bugs.
    Sadly, any coverage-improving test may simply execute code                       3.3     Hardening Tests
that does not matter while chasing the goal of elevating coverage.                   In this section, we formally define the concept of a hardening test:
The freshly covered code may not matter because, for example,                        one that is designed to pass on the current version of the system
it is never executed by any users. Even when a new test covers                       and catch future regressions.
previously uncovered code that does matter, it may not be executed                      This is the kind of test produced by many test generation systems
in a way that can reveal bugs. It is well known, from many empirical                 and approaches, including Symbolic Execution systems [17], such
studies, that increasing coverage does not provide a guarantee of                    as Klee [15], CUTE [75] and DART [32], Search Based Systems such
increasing fault revelation [30, 33, 49, 52]. In fact, evidence tends                as EvoStuite [31] and Austin [56], Mutation-based test generation
to suggest [19] that the best techniques to increase fault revelation                Systems such as Javalanche [74] and SHOM [35], Replay Systems
focus specifically on faults targeted by mutation testing [51].                      such as Fausta [61] and JRapture [77], Observation-based systems
FSE Companion ’25, June 23–28, 2025, Trondheim, Norway                                                                           Harman, O’Hearn, Sengupta


such as Carving [25], TestGen [9] and Pankti [78] and LLM-based               automatically provides an example of a fault that is guaranteed to
systems such as TestGen-LLM [5], CoverUp [70], HITS [83] and                  be caught by the new proposed test. As a result, ACH tests accepted
ACH [29].                                                                     by the engineer are strong hardening tests; the engineer confirms
   Although these systems have different notions of coverage, such            that the test correctly passes on the current revision, while the ACH
as mutants, serialised object observations and code, they all share           technology automatically generates the assurance that it catches at
the same approach to the oracle, which is that the behaviour of fu-           least one bug (the mutant) not caught by any existing test.
ture changes should not disrupt the previously observed behaviour.                A strong hardening test is also a weak hardening test, by def-
This is the ‘regression oracle’ [7], in which a previous revision of          inition, but not vice versa. When we refer to a ‘hardening test’
the system acts as the ground truth oracle for future revisions.              (without qualification) we mean a weak hardening test (that is, one
   A test 𝑡 for the revision 𝑅, is a weak hardening test, denoted             that may also happen to be strong). When we wish to refer to a
Hardening𝑅 (𝑡), if and only if it passes on 𝑅 and there exists a pos-         weak hardening test that is also not a strong hardening test, we call
sible next revision on which it fails when it should. More formally,          it a ‘strictly weak hardening’ test.
Definition 6 defines (weak) hardening.                                            Notice that a hardening test may be a coverage-improving test
                                                                              for some structural coverage criterion, such as statement coverage,
  Definition 6 ((Weak) Hardening Test). Hardening𝑅 (𝑡) ⇔
                                                                              but not necessarily so. The test may cover exactly the same code as
Passes𝑅 (𝑡) ∧ ∃𝑅 ′ .Parent(𝑅 ′ ) = 𝑅 ∧ Fails𝑅 ′ (𝑡) ∧ Oracle𝑅 ′ (𝑡) = fail.
                                                                              already covered, yet do so in a different way to any existing test.
   Definition 6 is ‘weak’ in the sense that it does not require that              A hardening test distinguishes the behaviour of the current ver-
the test, 𝑡 should correctly pass on 𝑅, merely that it should correctly       sion of the system from that of some future change. In the termi-
fail on some future revision, 𝑅 ′ . The test ‘hardens’ the code base          nology of mutation testing, where we think of a buggy 𝑅 ′ as a
against at least one buggy revision it might encounter in future.             mutant of 𝑅, a hardening test kills the mutant. It was this desire
   A test 𝑡 for revision 𝑅, is a strong hardening test denoted                to go beyond coverage to automatically generate hardening tests
SHardening𝑅 (𝑡), if it is a weak hardening test for 𝑅 and it passes           that led to our recent work on mutation-guided LLM-based test
correctly (true negative) for 𝑅. More formally, Definition 7 defines          generation [29]. ACH deployment guarantees, not only that tests
strong hardening.                                                             are strong hardening, but also that they can catch at least one bug
   Definition 7 ((Strong) Hardening Test).                                    that is uncaught by any existing test.
                                                                              Running example: Throughout the following definitions we will
      SHardening𝑅 (𝑡) ⇔ Hardening𝑅 (𝑡) ∧ Oracle𝑅 (𝑡) = pass                   present illustrative examples of tests and pull requests, based on a
                                 ⇔                                            hypothesised simple database application. This illustrative appli-
                             Passes𝑅 (𝑡)                                      cation involves a ‘product’ object that includes a list of supplier
                                 ∧                                            locations. This list should be initially empty. It should be a list type
                       Oracle𝑅 (𝑡) = pass                                     because there could be more than one supplier, and it should be
                                 ∧                                            empty initially because we do not initially have any suppliers when
                       ∃𝑅 ′ .Parent(𝑅 ′ ) = 𝑅                                 we create the object. Example 1 gives a unit test that we might write
                                 ∧                                            for this database application.
                             Fails𝑅 ′ (𝑡)
                                 ∧                                               Example 1 (Weak Hardening Test).
                                                                                 def weak_check_initial_supplier_locations():
                       Oracle𝑅 ′ (𝑡) = fail                                        product = Product()
                                                                                   locs = product.get_supplier_locs()
   A strong hardening test is one for which we know that the                       assertTrue (not locs)
passing signal on the current revision is a correct passing signal. In
general, this relies on oracle knowledge which cannot always be                  If a subsequent pull request were to introduce a bug that inadver-
known at test generation time; it may not even be well defined, since         tently assigns a nonempty list to the locations field on initialisation,
engineers may disagree, with the consequence there is no ground               Example 1 would catch that bug. However, suppose the constructor
truth. Nevertheless, if there is a known agreed oracle outcome                Product() already contains a bug (in the current revision) that al-
for the test case, then it would be possible to determine, at test            lows the supplier locations field to be null (i.e., None type in Python).
generation time, whether a weak hardening test is strong. Indeed,             In this situation, Example 1 is incorrectly passing on the current
in many cases it is worth the additional (minor) friction of asking           revision, due to the familiar problem in Python that the unary not
the engineer to check that the test is correct in passing on the              operator returns True when applied to the empty list, but also when
current revision. This human-in-the loop check ensures that a weak            applied to the null value None.
hardening test is also a strong hardening test, thereby allowing us              If the current revision contains this bug, then Example 1 is a
to proceed with greater confidence.                                           strictly weak hardening test (i.e., it is not strong hardening). We can
   This is what we have done with our deployment of TestGen-                  create a better test, Example 2 below, which is strong hardening.
LLM [5] and ACH [29], both of which generate tests using LLMs,                Example 2 fixes the assertion, replacing assertTrue (not locs)
and propose these to the engineer who checks that the behaviour of            with assertTrue (locs is not None and len(locs) == 0).
passing on the current version of the system is correct. In the case             Example 2 (Strong Hardening Test).
of TestGen-LLM the engineer also has to check whether the test will              def strong_check_initial_supplier_locations():
                                                                                   product = Product()
likely catch future regressions. This extra obligation places a further            locs = product.get_supplier_locs()
burden on the human engineer that is removed by ACH, since it                      assertTrue (locs is not None and len(locs) == 0)
Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges            FSE Companion ’25, June 23–28, 2025, Trondheim, Norway


   Strictly weak hardening tests are problematic because they in-                    check_foo that fails precisely when a call site to foo passes a null
troduce a false sense of security; the current revision is wrongly                   actual parameter.
believed to be more correct than it is. Furthermore, the same test
                                                                                        As the pull request in Example 4 below shows, it is almost impos-
weakness will extend to testing future possible revisions that may
                                                                                     sible for a test to attain perfect precision and recall. Example 4 does
contain the same bug, and which the strictly weak test would al-
                                                                                     not challenge the belief that Example 3 exhibits perfect recall, but
low to slip through as an avoidable false negative. This is why we
                                                                                     it makes it clear that it does not exhibit perfect precision. Once the
believe it is worth the minor friction inherent in proposing newly
                                                                                     pull request is submitted, it would become clear the check_foo’s
generated weak hardening tests to engineers to review (rather than
                                                                                     failure when the call site passes null to foo, is no longer a correct
simply automatically landing them into production without human
                                                                                     failure; it is a false positive. Since the revision in Example 4 exists
review).
                                                                                     (as a potential future revision), we must conclude that the precision
   The engineers’ reviews ensure that we check the oracle and make
                                                                                     of check_foo from Example 3 never was perfect.
the engineers the final arbiters of whether tests should land into
production. With this simple approach, we significantly increase                        Example 4 (A pull reqest that confirms that check_foo
the probability that the weak hardening tests we generate will prove                 does not have perfect precision). Consider a pull request for the
to be strong hardening tests when deployed.                                          function foo that adds a check on the actual parameter, and gracefully
                                                                                     behaves reasonably when that parameter should ever be null.
3.4     Hardening Tests’ Precision and Recall
                                                                                     3.5     Measuring Tests’ Precision and Recall
A test 𝑡 for revision 𝑅, is a Perfect Precision Hardening test denoted
PPHardening𝑅 (𝑡), if it is a hardening test for 𝑅 and, on any occasion               Although we cannot expect tests to have perfect precision and
when it fails, it is right to fail according to the oracle (no false                 recall, we can measure the precision of a hardening test over a
positives). More formally, Definition 8 defines a (Perfect Precision)                series of test outcomes. When measuring precision we tend to
Hardening Test.                                                                      use pseudo false positives [6] as our criterion, because when an
                                                                                     engineer rejects a test signal, even if it is a ‘correct’ signal, then
   Definition 8 ((Perfect Precision) Hardening Test).                                this represents the degree of friction, and that should be regarded
                 PPHardening𝑅 (𝑡) ⇔ Hardening𝑅 (𝑡)                                   as a kind of false positive; a pseudo false positive. The distinction
                                      ∧                                              between pseudo-false positive and false positive is an interesting
                 ∀𝑅 ′ .Fails𝑅 ′ (𝑡) ⇒ Oracle𝑅 ′ (𝑡) = fail                           illustration of the difference between practical software testing and
                                                                                     theoretical software testing.
  A test 𝑡 for revision 𝑅, is a Perfect Recall Hardening test denoted                    We can measure precision, but tests’ recall cannot be directly
PRHardening𝑅 (𝑡), if it is a hardening test for 𝑅 and, on any occasion               measured, because it relies on perfect knowledge about the ground-
when it should fail according to the oracle, it does fail (no false                  truth expected behaviour. Instead, we can estimate the recall using,
negatives). More formally, Definition 9 defines a (Perfect Recall)                   for example, mutation testing (to sample the space of potential
Hardening Test.                                                                      faults) or leak through of faults into production (to determine a
                                                                                     lower bound on the number of false negatives).
   Definition 9 ((Perfect Recall) Hardening Test).
                                                                                         Practicing software engineers are typically forced to care more
                 PRHardening𝑅 (𝑡) ⇔ Hardening𝑅 (𝑡)                                   about precision than recall. Perfect recall is unachievable, because
                                    ∧                                                perfect (i.e., exhaustive) testing is unachievable, and so recall is
                 Oracle𝑅 ′ (𝑡) = fail ⇒ ∀𝑅 ′ .Fails𝑅 ′ (𝑡)                           an aspiration. Perfect precision is also unachievable, because the
                                                                                     underlying questions are fundamentally undecidable in general.
    We can conjoin definitions 8 and 9 to produce a definition of a
                                                                                     Although we cannot hope achieve perfect precision and recall, un-
‘perfect precision and recall hardening test’. A perfect precision and
                                                                                     necessarily low precision can lead to the entire automated testing
recall hardening test passes on the current revision, and correctly
                                                                                     technology being abandoned due to friction on the engineers who
fails on every possible future buggy revision (and fails on no other
                                                                                     waste their time considering false positives. By contrast, unneces-
revision). That is, for every possible future revision, the test fails
                                                                                     sarily low recall is less risky to deployment. In this section, we want
if and only if the revision is buggy with respect to the property
                                                                                     to capture this inherent prioritization, and to suggest a different
tested.
                                                                                     way of measuring precision and recall.
    A perfect precision and recall test is an idealised test that may be
                                                                                         It is important to avoid measuring and goaling on precision
hard to achieve in practice. Example 3 illustrates how hard it is to
                                                                                     and recall independently. It is also important to avoid aggregates
achieve perfect precision and recall. It defines a scenario in which
                                                                                     such as the F1 score that implicitly treat the two measurements
a test may have almost perfect precision and recall. Example 3, is a
                                                                                     as convertible commodities. That is, by aggregating, we implicitly
test that might, at first, appear to have perfect precision and recall.
                                                                                     accept that it is possible to trade a certain amount of precision
It is certainly hard to construct an example where it incorrectly
                                                                                     degradation for a certain amount of recall improvement. This does
fails, or an example where it incorrectly passes, but not impossible.
                                                                                     not carry over well to the use case of industrial deployment of
   Example 3 (Near perfect precision and recall test:                                automated test generation.
check_foo). Suppose the function foo has a single nullable formal                        Rather, we propose to goal on the 𝑅@𝑃 = 𝑝. That is, the recall
parameter but its implementation assumes that the actual parame-                     achievable when the precision (𝑃) is fixed to be no lower than 𝑝.
ter passed is never null. For this example, we then write a unit test,               For example, 𝑅@𝑃 = 0.8 is the recall achievable when we demand
FSE Companion ’25, June 23–28, 2025, Trondheim, Norway                                                                  Harman, O’Hearn, Sengupta


a precision of at least 0.8, thereby ensuring that the ratio of false      a hardening test is a regression test; the current behaviour of the
positives to true positives is no worse than 1:4.                          system acts as an oracle (the ‘regression oracle’ [7]).
   Choosing 0.8 for 𝑝 is a conservative choice. We can be fairly sure         A test 𝑡 for revision 𝑅 is a Weak Regression Catching Test for
that, for most applications, this will be a sufficiently high precision:   𝑅, RCatching𝑅 (𝑡), if it is a weak catching test for 𝑅 and is also a
engineers will be prepared to tolerate friction 20% of the time, if        hardening test for the parent of 𝑅. More formally, Definition 12
the other 80% of cases throw up true positive catches.                     defines a Weak Regression Catching Test.
                                                                             Definition 12 (Weak Regression Catching Test).
3.6    Catching Tests
In this section, we define the notion of a catching test: one that fails                             RCatching𝑅 (𝑡)
on the current revision and may thereby reveal a bug.                                                     ⇔
   A weak catching test 𝑡, for revision 𝑅, denoted Catching𝑅 (𝑡), is                               PassesParent(𝑅) (𝑡)
simply a test that fails on 𝑅. More formally, Definition 10 defines a                                      ∧
(Weak) Catching Test.                                                       (∃𝑅 ′ .Parent(𝑅) = Parent(𝑅) ∧ Fails𝑅 ′ (𝑡) ∧ Oracle𝑅 ′ (𝑡) = fail)
                                                                                                           ∧
   Definition 10 ((Weak) Catching Test).                                                               Fails𝑅 (𝑡)
                       Catching𝑅 (𝑡) ⇔ Fails𝑅 (𝑡)                             A test 𝑡 for revision 𝑅 is a Strong Regression Catching Test for
                                                                           𝑅, SRCatching𝑅 (𝑡), if it is a strong catching test for 𝑅 and is also
   A weak catching test on some revision 𝑅 may give a false positive
                                                                           a hardening test for the parent of 𝑅. More formally, Definition 13
signal, so we define the concept of a strong catching test (that gives a
                                                                           defines a Strong Regression Catching Test.
true positive signal). A strong catching test 𝑡, for revision 𝑅, denoted
SCatching𝑅 (𝑡), is a weak catching test for which the failure on 𝑅 is        Definition 13 (Strong Regression Catching Test).
regarded as a true positive according to the oracle. More formally,
                                                                                                    SRCatching𝑅 (𝑡)
Definition 11 defines a (Strong) Catching Test.
                                                                                                           ⇔
   Definition 11 ((Strong) Catching Test).                                                         PassesParent(𝑅) (𝑡)
                                                                                                            ∧
          SCatching𝑅 (𝑡) ⇔ Fails𝑅 (𝑡) ∧ Oracle𝑅 (𝑡) = fail
                                                                            (∃𝑅 ′ .Parent(𝑅) = Parent(𝑅) ∧ Fails𝑅 ′ (𝑡) ∧ Oracle𝑅 ′ (𝑡) = fail)
  Example 5 is a pull request for which the hardening test from                                             ∧
Example 2 would subsquently also become a (strong) catching test.                                      Fails𝑅 (𝑡)
                                                                                                            ∧
   Example 5 (Strong Catching Test). Suppose a pull request                                        Oracle𝑅 (𝑡) = fail
inadvertently updates the code for the Product constructor to set the
supplier location to None.                                                   This simplifies to

   The test defined in Example 2 will correctly fail on the pull
                                                                           SRCatching𝑅 (𝑡) ⇔ PassesParent(𝑅) (𝑡)∧Fails𝑅 (𝑡)∧Oracle𝑅 (𝑡) = fail
request in Example 5, so we conclude that the test is a hardening
and (strong) catching test for the pull request in Example 5.                 Since the test defined in Example 2 is both hardening and (strong)
   As with weak and strong hardening tests, we refer to a weak             catching for the pull requests in Example 5, we conclude that it is a
catching test that is also not a strong catching test as a ‘strictly       (strong) regression catching test for this pull request.
weak catching’ test.
   In general, Search Based Software Testing (SBST) [36, 63] sys-          3.8    Functionality Catching Test
tems such as Austin [56] for C and EvoSuite [31] for Java, and test        A catching test does not necessarily need to be a regression catching
generation systems based on concolic execution [17, 32, 75] such           test. We want to push testing beyond the Regression Only Trap
as Klee [16] generate hardening tests. They seek to catch future           (ROT). The revision for which the test catches a bug may have
regressions, but use the current version as the oracle, so cannot          intended new functionality which the test catches. If this is new
catch new bugs, just regressions.                                          functionality, then the catching test should not pass on the parent.
   By contrast, automated fuzzing techniques [59], such as AFL [84]        It may even be that the catching test, 𝑡 for a revision, 𝑅 does not
and automated end-to-end test generation systems, such as                  even build on Parent(𝑅). For example, suppose 𝑅 introduces a new
Sapienz [6, 60], WW [3] Fausta [61], and Google’s RecSim [48]              method, 𝑚 and that 𝑡 tests the behaviour of 𝑚. Since 𝑚 does not
can potentially find Catching JiTTests. However, they use the im-          exist in Parent(𝑅), then 𝑡 will not build on Parent(𝑅).
plicit oracle (not the regression oracle) to determine whether the            We define a Functionality Catching test, as one that catches faults
test has failed. These Catching JiTTests can therefore catch, only         in 𝑅 but is not a hardening test for Parent(𝑅). As with regression
those truly egregious and obvious faults that are flagged by the           testing, it can be strong or weak depending on whether the failure
implicit oracle (such as memory violations and crashes).                   on 𝑅 concurs with the oracle.
                                                                              A Weak Functionality Catching test 𝑡, for revision 𝑅 is a weak
3.7    Regression Catching Test                                            catching test for 𝑅 that is not a hardening test for the parent of 𝑅.
It could be that a catching test for 𝑅 is also a hardening test for        More formally, Definition 14 defines a Weak Functionality Catching
the parent of 𝑅. We call this a ‘regression catching’ test, because        Test, FCatching𝑅 (𝑡).
Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges            FSE Companion ’25, June 23–28, 2025, Trondheim, Norway


    Definition 14 (Weak Functionality Catching Test).
                           FCatching𝑅 (𝑡)
                                  ⇔
              ¬(HardeningParent(𝑅) (𝑡)) ∧ Catching𝑅 (𝑡)
                                  ⇔
             (¬(BuildsParent(𝑅) (𝑡)) ∨ FailsParent(𝑅) (𝑡)∨
 ¬(∃𝑅 ′ .Parent(𝑅) = Parent(𝑅) ∧ Fails𝑅 ′ (𝑡) ∧ Oracle𝑅 ′ (𝑡) = fail))
                                  ∧
                              Fails𝑅 (𝑡)
    The definition of a weak functionality catching test is simple
and intuitive: it is one that is catching but not hardening. However,
                                                                                     Figure 1: Hardening and catching Venn Diagram: Strong is a
instantiating this simple definition, reveals several implicit criteria.
                                                                                     subset of weak. A test can be both (strong/weak) hardening
The test might not build on the parent, or it could build yet fail on
                                                                                     on parent and also (strong/weak) catching on child. Each
the parent. There is also the possibility that there does not exist a
                                                                                     region of the Venn diagram corresponds to an interesting
revision that shares the same parent and for which the test correctly
                                                                                     and important test behaviour.
fails on that revision. This last criterion arises due to the ‘weak’
nature of the catch.
    If we know that the test correctly fails on the current revision (a
‘strong’ catch), then this awkward criterion disappears. This can be                 diagram in Figure 1. In each of these eight regions there reside
seen in the definition of strong functionality catching test. A Strong               tests that exhibit interesting behaviours, five of which (the shaded
Functionality Catching test 𝑡, for revision 𝑅 is a strong catching                   regions) give rise to misleading test signals, while only three of
test for 𝑅 that is not a hardening test for the parent of 𝑅. More                    which lead to reliable true signal. In the remainder of this section
formally, Definition 15 defines a Strong Functionality Catching                      we explain the eight categories of test depicted in Figure 1.
Test, SFCatching𝑅 (𝑡).                                                               Five misleading classes of signal (shaded regions in Figure 1):
                                                                                     A test that is strictly weak hardening but not catching (Region
    Definition 15 (Strong Functionality Catching Test).                              1) suggests that all is well, when it is not. A test that is strictly
                           SFCatching𝑅 (𝑡)                                           weak catching but not hardening (Region 7) may appear to catch
                                  ⇔                                                  new functionality but, because it is strictly weak catching, it is a
              ¬(HardeningParent(𝑅) (𝑡)) ∧ SCatching𝑅 (𝑡)                             ‘fake’ functionality catching test. The remaining three misleading
                                  ⇔                                                  signals include two combinations of strictly weak and strong hard-
              (¬(BuildsParent(𝑅) (𝑡)) ∨ FailsParent(𝑅) (𝑡))                          ening/catching (Regions 4 and 5), and finally, the most misleading
                                   ∧                                                 of all (Region 3): both strictly weak hardening and strictly weak
                   Fails𝑅 (𝑡) ∧ Oracle𝑅 (𝑡) = fail                                   catching. A test that resides in this most misleading category might
                                                                                     fail to catch a bug and subsequently ‘complain’ (i.e., incorrectly fail)
   As an example, suppose a pull request updates the Product                         when it is fixed.
object to introduce a method, distance, that computes the distance                   Three classes of true signal: A test that is strong hardening, but
to each supplier. Suppose further that the pull request contains a                   not catching (Region 2), is simply a good hardening test that has yet
bug that incorrectly omits an abs operator that is required to ensure                to catch anything. A test that is strong catching but not hardening
that differences in location are always positive. Example 6 is a strong              (Region 8) is a true functionality catching test. A strong catching
functionality catching test. That is, it will not build on the parent                test that is also a strong hardening (Region 6) is a true regression
of the pull request (since it calls distance, which does not exist                   catching test.
in the parent). It also fails correctly on the pull request, since the               Research Challenge 1: find techniques that can help reduce the
distance is incorrectly computed as a negative number.                               risk of miss-classification of tests according to this classification.
    Example 6 (Strong Functionality Catching Test).
    def distance_test():
      test_supplier = MOCK_PRODUCT(Locations.LONDON)
                                                                                     5     DEPLOYMENT OF HARDENING AND
      dist = test_supplier.distance()                                                      CATCHING TESTS
      assertTrue(dist>=0)
                                                                                     In this section we review the options for deploying hardening and
   Now, suppose that the author of the pull request fixes the bug                    catching tests. In the case of timely hardening, this provides a way
as a result of the failure of the test in Example 6. Now Example 6                   to formally analyse the (current widely-adopted) approach used in
correctly passes on the fixed pull request. As such, the test now                    research and practice for automated regression test generation. For
becomes a (strong) hardening test for the pull request revision,                     JiTTests, the analysis reveals the complexities and trade-offs in the
protecting it against future regressions in the distance method.                     decision processes, leading to an initial set of proposed deployment
                                                                                     options. We do not claim these proposed deployment options are
4    EIGHT CATEGORIES OF TEST                                                        unequivocal nor that they are comprehensive but rather, that they
The 8 possible combinations of strong/weak hardening on parent                       illustrate how the formal analysis together with a case-by-case
and catching on child and their intersections are shown in the Venn                  reasoning can shed light on the trade-offs involved.
FSE Companion ’25, June 23–28, 2025, Trondheim, Norway                                                                                    Harman, O’Hearn, Sengupta


5.1     Cases of Behaviour for Timely Hardening                                            Deploying Hardening Timely Tests: Based on Table 1, when
        Tests                                                                              we generate a hardening test from a current revision, and the test
                                                                                           passes, we will offer the test up for review by an engineer who
Table 1 captures3 the possible combinations of outcomes of a test
                                                                                           may choose to land it into production. At the heart of this decision,
generated on a revision, which we call the parent, and that seeks to
                                                                                           resides the core belief in the value of testing itself:
generate a hardening timely test for this revision. Such a test seeks
to harden the parent against future regressions in a subsequent                                   “The risk of future false positives is outweighed by
revision, which we call the child. In this mode of deployment, we                                 the chance of catching future real bugs.”
generate tests on the parent pull request, where the test passes
on this parent, and we have some unknown future children, for                              Research Challenge 2: automatically distinguish weak from
which we want to protect the code. From this case-by-case analysis,                        strong hardening: find techniques that can take partial oracle in-
we can see that any test that passes on the current version of the                         formation and use this to improve the chance that a weak hardening
system is worthy of consideration.                                                         test is strong, to further reduce human oracle effort.
   In order to distinguish between strictly weak and strong harden-                        Research Challenge 3: automatically distinguish regression
ing tests, we can use the simple expedient of reporting the generated                      catching from false positive: find techniques that reduce the
test to the engineer. Indeed, this is what we currently do with the                        risk that a test will raise false positive signal, due to strictly weak
ACH hardening tool [29]. It is also what many other other test                             catching.
generation tools do. Reporting the test to the engineer is not neces-
sary: one could simply auto-land all passing generated tests into                          5.2    Cases of Behaviour for JiTTests
production, but such auto landing would be risky. It risks landing
                                                                                           We are specifically interested in the case of generated JiTTests,
strictly weak hardening tests, or even tests that are not hardening
                                                                                           because a JiTTest is generated at the last occasion in which it is
at all because they can catch no future regression.
                                                                                           possible to catch a bug before it lands into production. This makes
Actionable Conclusion: By seeking the engineer’s review, we
                                                                                           this particular testing paradigm highly important and impactful.
eliminate three (of the five) problematic categories (regions 1, 3 and
                                                                                           Despite this importance, there has been very little discussion and
5 deplicted in Figure 1), thereby dramatically reducing potential
                                                                                           analysis of the opportunities and challenges for generation of such
future confusion from ‘fake’ test signals. Furthermore, the effort
                                                                                           JiTTests. Table 2 collects the possible outcomes for attempts to gen-
required from the engineer is low; they merely need to determine
                                                                                           erate a JiTTest for a revision 𝑅, and the consequences of accepting
whether the assertion is reasonable, and therefore the test is right
                                                                                           the signal from the generated test.
to pass on the current revision. This is a good example the principal
                                                                                              For a JiTTest, the test generation tool has available to it, both the
that
                                                                                           pull request, 𝑅, and its parent, Parent(𝑅), at test generation time.
        “A relatively small amount of timely oracle informa-                               The table groups the analysis results for the current revision and
        tion can dramatically enhance test effectiveness.”                                 its parent into five categories, with a sub-table for each category.
   In this case, a check that may take an engineer seconds to per-                         In this section, we consider each category and turn.
form removes three fifths of the problematic test categories. Rather                       Category 1 (Parent Hardening): tests that pass on the parent.
than thinking of the oracle as some unknowable theoretical con-                            A hardening test may have been generated some months or even
struct, there is a more ‘engineering orientated viewpoint’: The                            years before the first time it fails in production on a new revision.
oracle is not a discrete logical construct, but better thought of as a                     By contrast, a JiTTest is generated on the fly when a revision is
continuous resource on which we can draw. Since it is an expen-                            submitted. Where the test passes on the parent revision, the JiTTest
sive resource on which to draw, we need to make the engineering                            may be a hardening test for the parent, although it is not timely for
trade-off that determines when the cost is worth the benefit. The                          the parent, only for the child, since it is a JiTTest.
only situation where auto-landing generated tests might be safe                               Where the test passes on the parent, the case-by-case analysis is
from these problematic categories, is when tests are disposable.                           isomorphic to the case-by-case analysis for the hardening timely
For example, where they are generated to detect any changed be-                            test. This is shown in the first eight rows of Table 2. Although iso-
haviour after some a specific meaning-preserving refactoring, and                          morphic, the consequences of false positives are more problematic
subsequently discarded.                                                                    than for hardening timely tests, precisely because of the just-in-time
   Overall, therefore, we think it is safe to deploy hardening test                        nature of the test generation process.
generation and to land into production any tests that pass consis-                            Since the tests are generated just-in-time for the revision, it is
tently (i.e., non flakily [20, 39, 57]) on the current revision, that are                  likely that there is no existing test that fails. If there were, then we
acceptable to human reviewers. While this can have problems, as                            would not need to generate any further tests. Furthermore, there
indicated by the four problematic rows attributed to false negatives                       may simply be no existing tests at all (either passing or failing).
and false positives in Table 1, any such risks are merely specific                         This is precisely the scenario in which we would need to generate
instances of the risks that already pertain to human-written tests.                        JiTTests, to either increase confidence in the revision under test or
                                                                                           stop a bug form landing into production. For the same reason, when
3 The tables in this paper use a traffic light colour-coding scheme which is best viewed   a JiTTest fails on a revision, it may be the only test that provides
in colour, rather than in black-and-white. Green indicates a positive test experience,     any signal, whether passing or failing. Therefore, the risk of high
in which there is no friction from misleading signal and there is test benefit. Red
denotes friction: the test does not catch bugs but rather slows down the engineering       friction from a newly created false positive is considerably higher
development. Amber indicates possible friction, that is deemed to be probably tolerable.   than for a long-standing hardening timely test.
Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges           FSE Companion ’25, June 23–28, 2025, Trondheim, Norway


Table 1: The possible outcomes for generated timely tests. The test is generated for the current revision (the parent). We
distinguish the possible outcomes for the execution on some future child of the current revision. In describing the outcome for
a child we are also describing the outcome for some future ‘child of a child of a ...’ for some arbitrary number of revisions for
which the test passes. A test that fails on the child is one of the catching cases from Figure 1, whereas a test that passes on both
the parent and child transitions from one hardening case (for parent) to another (for the child), hence the ‘→’ in column 6.

     outcome on                oracle for         type                In
 parent    child            parent child          for child           Figure 1 Consequences if we were to accept the generated test’s signal
                 When tests were landed to harden the parent and now they are executed on some future child ...
 pass          pass    pass    pass    true negative 2 → 2 Strong Hardening test passes: All is well; move fast
 pass          pass    fail    pass    true negative 1 → 2 Strictly Weak Hardening test passes: the child fixes an uncaught bug
 pass          fail    pass    fail    true positive     6     True regression catching: Catches the child’s regression
 pass          fail    fail    fail    true positive     5     Aparent regression is more likely to be a Failed error propagation [12]
 pass          pass    fail    fail    false negative 1 → 1 Strictly Weak Hardening test continues to miss a bug
 pass          pass    pass    fail    false negative 2 → 1 Strong Hardening test misses a bug introduced by the child
 pass          fail    pass    pass    false positive    4     Strong Hardening test wrongly blocks the child
 pass          fail    fail    pass    false positive    3     Strictly Weak Hardening test wrongly flagged a fix as a new failure
                    When tests were landed to harden the parent, but some future child makes them obsolete ...
 pass          no build pass    —       —              2 → ⊥ Strong Hardening test retired: child removes tested functionality
 pass          no build fail    —       —              1 → ⊥ Strictly Weak Hardening test retired: Test was weak anyway


   We may choose to suppress the signal of a JiTTest to avoid undue                  It is safe to give this timely hardening signal to the engineer when
friction. The balance between risk and benefit will likely be deter-                 the test is generated for 𝑅.
mined differently by different organisations and different projects                  Category 3 (New Functionality Tests): tests that fail to build
within an organisation. Notwithstanding these organizational pref-                   on the parent. Where a JiTTest fails to build on the parent of the
erences, a general principle applies to all: the risk of friction is                 current revision and passes on the current revision, this is perfectly
higher for JiTTests than for merely timely tests.                                    safe to land (as the case analysis in Table 2 shows). Such a test
Actionable Conclusion: As a result of this analysis, a failing                       will become a new hardening test which, like others, may ‘bake in’
JiTTest signal may be regarded as a ‘trigger’ to do further testing or               wrong functionality, but there is the hope that other tests will catch
analysis before troubling the engineer with the signal. By contrast                  that. Furthermore, since the test fails to build on the parent, it is
passing JiTTest signal still plays the role of giving the engineer                   likely that this test is testing precisely the new functionality added
additional confidence, and such a passing JiTTest may also be a                      by the revision. Once again, as with Category 2 (for tests that fail
hardening JiTTest. It therefore makes sense to have a deployment                     on the parent), the just-in-time nature of the test gives extra signal
route for JiTTests, if only to create hardening JiTTests as the side                 because it does not build on the parent.
effect of attempting to generate catching JiTTests.                                  Actionable Conclusion: From the analysis of Categories 2 and 3,
Category 2 (Child Hardening): tests that fail on the parent                          we conclude that it is always wise to run a generated test on the
and pass on the child. For all four cases in Category 2, the JiTTest                 parent of a revision (to obtain this extra signal). This is not some-
fails on the parent but passes on the current revision. In any such                  thing that current test tools tend to do, but our analysis suggests
case, it is relatively safe to give the signal to the engineer. A Cate-              that it is not only worthwhile. Test generation might specifically
gory 2 test that is a hardening JiTTest is, by definition, both hard-                target the generation of such tests, and check/report test behavior
ening and timely for 𝑅, although it is neither hardening nor timely                  on the parent of the tested revision.
for its parent.                                                                      Category 4 (Cannot Land): tests that fail to build on the child.
   When the test is a true negative for the current revision, its                    Where a JiTTest fails to build on the current revision, it clearly
behaviour not only generates a new hardening test for the future                     cannot be landed as a new test. Nevertheless, its signal may be
(as a hardening timely test would), it can yield additional signal                   useful to the engineer. For example, if it previously failed on the
value since it is just-in-time. For example, the test may show that                  parent, then it may indicate a fix or an improved functionality in
the developer’s change has fixed (or improved) the behaviour of the                  the current revision. The engineer may even choose to cite the
parent revision. Such fail → pass JiTTests therefore add value over                  generated test as documentation for the improvement that their
and above purely generating new hardening timely tests (which                        revision achieves.
they also do ‘for free’). Also, fortunately, when the test is a false                Actionable Conclusion: From the analysis of Category 4, we
negative pass, the negative consequences are relatively low, as the                  conclude that we cannot land tests that fail to build on the child,
case-by-case analysis reveals. This observation mimics the analysis                  but it is always worth giving the signal to the engineer.
for hardening timely tests in Section 5.1.                                           Category 5 (Likely Discard): tests that fail on both the parent
Actionable Conclusion: For both Categories 1 and 2 the generated                     and child. Finally, JiTTests that fail on both the parent and the
test is a hardening timely test for the child but not for the parent.                current revision will typically have to be discarded. The only corner
FSE Companion ’25, June 23–28, 2025, Trondheim, Norway                                                                 Harman, O’Hearn, Sengupta


Table 2: The possible outcomes for generation of Just-in-Time Tests (JiTTests). In this mode of deployment, tests are generated
on the fly when a revision is first submitted. A test is generated when a revision 𝑅 is submitted, based on the revision 𝑅, and
its parent, Parent(𝑅). The outcome of test execution on both the revision and the parent can be taken into account when
determining whether to land the test, report its signal (but not land the test), or to discard it completely. In Column 6, JiTTests
that fail on 𝑅 correspond to one of the categoiries from Figure 1, for which a decision must be made about whther to give the
signal to the engineer. Cases that do not fail on 𝑅 may correspond to transitions between categories (Parent(𝑅) → 𝑅).

     outcome on                   oracle for             type        In
 Parent(𝑅) 𝑅                 Parent(𝑅) 𝑅                 for 𝑅       Figure 1 Consequences of landing the JiTTest and reporting its signal
                                        Category 1 (Parent Hardening): tests that pass on the parent ...
 pass          pass          pass         pass      true negative 2 → 2 Strong Hardening JiTTest hardens 𝑅
 pass          pass          fail         pass      true negative 1 → 2 Strictly Weak Hardening on parent (missed bug likely fixed)
 pass          fail          pass         fail      true positive    6      True Regression catching JiTTest
 pass          fail          fail         fail      true positive    5      Strictly Weak Hardening JiTTest
 pass          pass          fail         fail      false negative 1 → 1 Strictly Weak Hardening JiTTest misses a long-standing issue
 pass          pass          pass         fail      false negative 2 → 1 Strong Hardening JiTTest missed a bug introduced by 𝑅
 pass          fail          pass         pass      false positive   4      Strictly Weak Catching JiTTest wrongly blocks change
 pass          fail          fail         pass      false positive   3      Doubly Weak JiTTest wrongly blocks a fix
                           Category 2 (Child Hardening): tests that fail on the parent and pass on the child ...
 fail          pass         pass       pass     true negative 7 → 2 Strong Hardening JiTTest: 𝑅 fixes a broken test
 fail          pass         fail       pass     true negative 8 → 2 Strong Hardening JiTTest correctly identifies a fix
 fail          pass         pass       fail     false negative 7 → 1 Strictly Weak Hardening JiTTest lets through new bug
 fail          pass         fail       fail     false negative 8 → 1 Strictly Weak Hardening JiTTest lets an existing bug remain
                                 Category 3 (New Functionality Tests): tests that fail to build on the parent ...
 no build      pass          —           pass      true negative ⊥ → 2 Strong Hardening for new functionality introduced by 𝑅
 no build      fail          —           fail      true positive     8       Strong Hardening/Catches 𝑅’s new functionality bug
 no build      pass          —           fail      false negative ⊥ → 1 Strictly Weak Hardening bakes in incorrect new functionality
 no build      fail          —           pass      false positive    7       Strictly Weak Catching blocks a perfectly good change
                                     Category 4 (Cannot Land): tests that fail to build on the child ...
     outcome on                   oracle for      type
 Parent(𝑅) 𝑅                 Parent(𝑅) 𝑅          for 𝑅                   Consequences of reporting the JiTTest signal to the engineer
 pass      no build          pass        —        —             2 → ⊥ Strong Hardening but tested functionality is removed by 𝑅
 fail      no build          fail        —        —             8 → ⊥ Strong Catching JiTTest may flag the bug removed by 𝑅
 fail      no build          pass        —        —             7 → ⊥ Strictly Weak Catching false positive, but it is removed
 pass      no build          fail        —        —             1 → ⊥ Strictly Weak Hardening weakness means little valuable signal
                                Category 5 (Likely Discard): tests that fail on both the parent and child ...
     outcome on                   oracle for      type
 Parent(𝑅) 𝑅                 Parent(𝑅) 𝑅          for 𝑅                      Consequences of discarding the JiTTest and ignoring its signal
 fail      fail              pass        pass     false positive     7       Strictly Weak Catching JiTTest should be discarded
 fail      fail              fail        pass     false positive     7       Strictly Weak Catching so would have rejecting a correct fix
 fail      fail              pass        fail     true positive      8       Discarding Strong Catching JiTTest meant missing a new bug
 fail      fail              fail        fail     true positive      8       Discarding Strong Catching means missing existing bug


case is one where we believe we have discovered a long-standing            conclusions from the analysis in this section. Futhure research will
or new bug, but there will need to be an additional way to gain            hopefully find ways to distinguish sub-categories; one aspect of the
confidence that this is what the test is revealing rather than a false     Catching JiTTest challenge (see Section 6).
positive.                                                                  Deploying catching tests: We summarize the foregoing actionable
   The risk of friction from a false positive is relatively high. The      conclusions to document the eight possible outcomes that can be
chances of finding such a bug with a test that fails on the parent         observed, irrespective of the oracle in Table 3.
and the current revision is low. Therefore, it seems likely that the
balance probabilities is in favour of discarding all such tests.
Actionable Conclusion: From the analysis of Category 5, without
                                                                           6    RESEARCH CHALLENGE 4: THE CATCHING
new techniques, we may sadly have to discard tests that fail on both            JITTEST CHALLENGE
the parent and the child. This is the least certain of our actionable      Earlier in the paper, we discussed Research Challenges 1 – 3, which
                                                                           concerned classification and hardening tests. These three challenges
Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges            FSE Companion ’25, June 23–28, 2025, Trondheim, Norway


Table 3: Suggested JiTTest deployment approach based on signal from parent and current revision. Based on the case by case
analysis in Table 2 we give a proposed deployment approach for each case. Of course, unlike the analysis from Table 2, oracle
information is assumed to be unavailable, and therefore we do not know whether signal is a true/false positive/negative.

                                      Category 1: When JiTTest passes on parent of current revisions ...
     outcome on
 Parent(𝑅) 𝑅                  Decision           Possible undesirable consequences                      Best case consequences
 pass      pass               Land Test          Bakes in long standing issue                           Catches future regressions
 pass      fail               Report Fail        False positive; but other tests may pass               Catches a current regression
                                 Category 2: When JiTTest fails on parent but passes on current revision ...
     outcome on
 Parent(𝑅) 𝑅                  Decision           Worst case consequences of landing new test            Best case consequences of landing new test
 fail      pass               Land Test          Bakes in newly created issue                           Catches future regressions in new functionality
                             Category 3: When JiTTest does not build on the parent of the current revision ...
     outcome on
 Parent(𝑅) 𝑅                  Decision           Worst case consequences                                Best case consequences
 no build  pass               Land Test          Bakes in newly created issue                           Catches future regressions in new functionality
 no build  fail               Report Fail        False positive; but other tests may pass               Catches bug in new functionality
                  Category 4: When JiTTest cannot land but may still give useful signal on the revision ...
     outcome on
 Parent(𝑅) 𝑅                       Worst case consequences of reporting signal  Best case consequences of reporting signal
 pass      no build   Give Signal  Wrongly claims that parent was correct       Confirms that behaviour was removed
 fail      no build   Give Signal  Wrongly claims that parent was incorrect     Helps to document a fix
                                   Category 5: When JiTTest fails on the current revision and its parent ...
     outcome on
 Parent(𝑅) 𝑅                                     Worst case consequences of discarding JiTTest          Best case consequences of discarding JiTTest
 fail      fail               Discard Test       Loses a bug catch on current revision                  Loses a potentially misleading test


are important, but the most challenging of all (and likely the most                  6.1     The Parsimonious Pull Request Problem
impactful) is Research Challenge 4: The Catching JiTTest Challenge,                          and the Oracle Scavenging Solution
which we summarise as follows:
                                                                                     As an extreme example of a parsimonious pull request, consider
                                                                                     the following hypothesised (but entirely plausible) pull request:
       “Given a buggy pull request and its parent, automati-
       cally generate a test that can reveal bugs in the pull                        title:     add location details to lookup query
       request, with low risk of false positive failure on non-                      summary: TSIA
       buggy pull requests. ”                                                        Test Plan: YOLO

                                                                                        Let us call this pull request 𝑇 𝑆𝐼𝐴. The acronym ‘TSIA’ stands for
   We call this the ‘Catching JiTTest Challenge’. It is essentially to               ‘Title Says It All’, while the acronym ‘YOLO’ stands for ‘You Only
improve the value of JiTTest R@P=𝑝 (See Section 3.5), for some                       Live Once’. Engineers may believe this level of detail is sufficient
suitably high choice of precision threshold 𝑝, such as 0.8.                          for the reviewer. Therefore, it is also the only information available
   Suppose a JiTTest, 𝑡, generated just-in-time for a revision, 𝑅,                   to any automatic technology seeking to design tests.
passes on 𝑅 and also happens to pass on Parent(𝑅). This test, 𝑡, is                  Code Summarization May Prove Insufficient: Of course, we
simply a hardening JiTTest (for 𝑅). However, if 𝑡 fails (or fails to                 could seek to use LLMs for code summarization [86] to provide a
build) on Parent(𝑅), then 𝑡 is a specific new kind of test case that                 better description in the summary. There has been much research
can be found only using the Just-in-Time paradigm: It could not                      work on this problem [26], and it would undoubtedly help the
be landed at any time before 𝑅 has landed, because it would land a                   human reviewer. Nevertheless, when our goal is to automatically
failing test signal or even break the build.                                         provide a better test plan than ‘YOLO’, we would need to be cautious
   The Catching JiTTest Challenge thus opens up a whole new class                    to ensure that our LLM-based test generation techniques do not
of test cases that can be generated. In order to tackle the challenge,               become victims to the obvious circularity involved by also using
we are inherently faced with a particularly pernicious example                       LLMs to infer the code summaries.
of the Oracle Problem: the only oracle information may be that                       Grounds for Optimism: Plenty of Computation Time: As
in the pull request itself. Such information can be exceptionally                    revealed in Section 2.4, the timing constraints for online test gener-
parsimonious.                                                                        ation are considerably more relaxed than might, at first, be thought.
FSE Companion ’25, June 23–28, 2025, Trondheim, Norway                                                                    Harman, O’Hearn, Sengupta


In deployment to continuous integration we are typically afforded          identical in the generated test case. In this situation, the comment
approximately eight hours to generate JiTTests. This allows for            that immediately proceeds the computation of distance in the pull
considerable static and dynamic analysis, and other automated              request is a highly relevant clue. An automated test technique,
techniques to be deployed to determine what signal, if any, should         especially one with the power of Large Language Model, ought to
be given to the pull request reviewer.                                     be able to infer, from this clue, that 𝐹𝐶 𝐽 is failing precisely because
Grounds for Optimism: Tackling the Oracle with LLMs: In the                it does not respect the assumption captured by this comment.
remainder of this section, we give two examples that illustrate ways           Notice that this does not mean that we should not report any
in which, even with a highly parsimonious pull request, we are able        signal to the engineer. Rather, it would be better to construct a
to glean sufficient information to weed out false positives in both        comment on the pull request that observes that the test fails because
putative regression catching and functionality catching JiTTests.          the assumption is not met.
                                                                           Test signal refinement → Test Improvement: Furthermore, the
6.2     Weeding out Fake Regression Catching                               tool could transform the test into a passing test that documents
Suppose we have a hardening JiTTest, 𝑅𝐶 𝐽 , that we are considering        the assumption by asserting that the distance computation will
as a potential Regression Catching JiTTest.                                raise a divide-by-zero exception when the current and destination
    Since we consider it to be a regression catching test, then, by        location are identical. This might encourage the engineer to make
Definition 12, 𝑅𝐶 𝐽 passes on Parent(𝑇 𝑆𝐼𝐴) and fails on 𝑇 𝑆𝐼𝐴. Now        an improvement that avoids an exception.
suppose that 𝑅𝐶 𝐽 asserts that the result of a query in the executed       Test Improvement → Automated Repair: The automated system
code returns a NULL field for the location column. Such a test is          could also suggest such an improvement: test generation thereby
seeking to enforce precisely the behaviour that we might reasonably        leading to improvement/repair suggestion.
infer 𝑇 𝑆𝐼𝐴 is seeking to extend/change. If we present this test to        LLM + non-executable text = Oracle: As these two simple ex-
the pull request author as a ‘catch’ we are likely to significantly        amples illustrate, the Catching JiTTest challenge is demanding,
increase the friction in their development experience. They will be        but not insurmountably so. In particular, we believe that LLMs’
understandably frustrated that their pull request, title ‘said it all’,    ability to reason with both code and natural language (in the same
and our generated test paid little attention to this.                      inference) provides a perfect bridge between the code and any avail-
    Even with this extremely parsimonious pull request, there is           able oracle information in non-executable text such as comments
enough information in the title alone to automatically determine           and other documentation, parsimonious though it may be. We see
that 𝑅𝐶 𝐽 should be discarded, and its signal not passed to the engi-      grounds for optimism in the use of ‘oracle scavenging’; the hunt for
neer (unless it is to confirm that they have changed the functionality     non-executable code hints to expected behaviour. There has been
in the way they intended). Therefore, an automated technique would         much recent work on LLMs for testing [26, 82], including recent
be able to identify that the 𝑅𝐶 𝐽 test failure is likely to be a false     work on the oracle inference problem [27, 47, 54, 64], but their
positive, even with only those six words of the 𝑇 𝑆𝐼𝐴 pull request         potential to combine with test generation to tackle the Catching
title and the behaviour of 𝑅𝐶 𝐽 as guidance.                               JiTTest Challenge remains currently open and untackled, yet highly
                                                                           impactful.
6.3     Weeding out Fake Functionality Catching
Suppose the code that implements the parsimonious pull request
                                                                           6.4    Oracle Scavenging for Non-executable Code
𝑇 𝑆𝐼𝐴 includes logic that calculates the fuel consumed per distance        Non-executable text is primarily authored by humans, and it will
travelled, as a report on fuel efficiency. Suppose that this logic is      increasingly become the way in which humans drizzle oracle in-
computed in terms of the location, and that the code in the pull           formation into software. This human author is arguably the best
request, relevant to this computation, is as follows.                      author for such documentation, because human authors can pro-
# Fuel efficiency computation assumes the destination                      vide these crucial fragments of oracle information. As discussed in
# location cannot be the same as the current location                      Section 3.2, Software Engineering is an inherently social process.
distance = distance(current_location, destination)
fuel_efficiency = fuel_consumed/distance                                   The oracle is one of the key points at which the social nature of
                                                                           the engineering discipline impinges. Different humans may even
   Suppose further that the pull request also includes the imple-
                                                                           disagree about what the expected behaviour of the software system
mentation of the new function distance.
                                                                           should be.
   Now suppose that we automatically generate a test 𝐹𝐶 𝐽 that,
                                                                              The oracle may change over time, even with the same engineer
on the face of it, appears to be a functionality catching JiTTest. By
                                                                           changing their view of expected behaviour over time. Regulatory
Definition 14, this means that 𝐹𝐶 𝐽 fails to build (or builds yet fails)
                                                                           and other changes may also impact the expected behaviour of soft-
on the parent, and also fails on the pull request. The new test, 𝐹𝐶 𝐽 ,
                                                                           ware. Therefore, without any further change in the code, the oracle
calls the new logic to determine the distance and, therefore, does
                                                                           may change. It will become increasingly important, as language
not build on the parent. So far, so good: the test appears to be a
                                                                           models play an increasingly wide role in code generation, to effi-
functionality catching JiTTest.
                                                                           ciently and accurately capture oracle information.
   Now comes the challenge: in order to be sure that the failure
                                                                              As the charter for the Source Code Analysis and Manipulation
on 𝑇 𝑆𝐼𝐴 is a true positive, we need to scavenge as much oracle
                                                                           workshop points out:
information as possible. Suppose 𝐹𝐶 𝐽 fails with a divide by zero
exception in the computation of fuel_efficiency, and that it does                 “Source code contains the only precise description of
this because the current and destination locations turn out to be                 the behaviour of the system” [10].
Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges           FSE Companion ’25, June 23–28, 2025, Trondheim, Norway


    This makes source code analysis and manipulation extremely                       𝑡, is generated for the pull request. The test 𝑡 will thereby reveal a
important, and it always will [34], but it also highlights what source               latent bug in 𝑚, that has already landed into production.
code cannot do, as well as what it can do. It is the only precise de-                Oracle Scavenging: The challenge of finding latent bugs with
scription of behaviour the system, but it is only that: a description                JiTTests lies primarily in locating additional sources of oracle in-
of the behaviour of the system, not a description of its intended                    formation that help to determine what a re-inserted method (or
behaviour. For the purpose of oracle extraction, we have to treat ex-                code fragment) 𝑚 is supposed to do. When we seek to find latent
ecutable text (the executable source code) as potentially suspicious,                bugs we have the most parsimonious pull request of all, from which
and focus on extraction of intended semantics from non-executable                    we can scavenge no new non-executable text. Fortunately, even
text, so that we can compare intended and actual semantics. This                     for this most challenging form of oracle scavenging, we can still
non-executable text may also be suspicious, of course, but at least                  automatically scour the continuous integration history for relevant
it provides an alternative view of expected behaviour. In the near                   past pull requests, extracting previous non-executable text from
future, there will be equal/greater importance for non-executable                    them. We can also seek out other documentation in the form of
system descriptions, so programming languages will need to evolve                    comments and other sources of non-executable text. Indeed, this
to support this.                                                                     new world of LLM-based code generation (and test generation)
    Perhaps we may even see a resurgence of interest in literate pro-                dramatically increases the importance of such non-executable text:
gramming [53]. In literate programming, the aim of non-executable                    it is no longer just written for human consumption, it must also be
text was to describe how the executable code performs its task.                      written for machine consumption.
Programming languages will need to evolve further to also include                        No matter what way we seek to incorporate oracle information
at least some informal specification of what those tasks are; what                   into the test generation approach, we will be able to use JiTTest
is the intended behaviour of the system. We find ourselves back                      generation as an approach to uncover latent faults in existing soft-
in previously well-trodden territory: agonizing over the absence                     ware systems. We can simply apply the delete-and-re-insert process
of specifications and the relationship between specification and                     arbitrarily many times to walk over a code base, using JiTTest gen-
testing. Formal specifications can help with test generation [43],                   eration to uncover latent bugs.
but the kind of oracle information/specification required need not
be formal. Furthermore, the uncertainty about the true intended                      7     CONCLUSIONS AND FUTURE WORK
behaviour suggests that it would not even be desirable to have to                    This paper explores the complex interplay between test generation,
rely on a purely formal specification.                                               test deployment and test oracles. Previous work has tended to focus
                                                                                     solely on test generation, and the regression oracle. We show that
                                                                                     modes of test deployment have a profound impact on the way
6.5     JiTTests Can Also Find Latent Bugs                                           we seek to automatically generate tests and report their signal
Many organisations have large legacy code bases and systems that                     to engineers. Our analysis also reveals the impact of taking non-
contain latent bugs that have resided in the code base, sometimes                    regression oracle fragments into account.
for years. By definition, such latent bugs are not mission-critical,                    Our primary contribution is to formally define and crystallise the
since they have survived some time without correction. Never-                        research challenges that emerge from this three-way interplay. We
theless, although not mission-critical, they may, collectively, pose                 hope that the paper clarifies the impact that this research agenda
significant problems for users and can also unnecessarily drain                      can have on industrial software testing. Our aim is to stimulate
resources and computational capacity.                                                the research community to take up and investigate the challenges
   Furthermore, some of these latent bugs may currently benefit                      raised herein, underpinned by precise formal definitions of the
from failed error propagation [12]. In this failed error propaga-                    challenges involved, and motivated by industrial experience that
tion situation, the bug is executed and ‘infects’ the local state of                 highlights their potential impact.
computation, but this infection never manifests itself as a failure,
because the bug is suppressed along all paths between the infected                   ACKNOWLEDGEMENTS
state and some point at which it becomes observable. Such failed                     We would like to thank the leadership of Meta’s Product Compliance
error propagation scenarios are vulnerable to future pull requests,                  and Privacy, Fundamental Artificial Intelligence Research (FAIR),
that may inadvertently unblock a path from the infection point to                    Developer Infrastructure (DevInfra), and Instagram Product Foun-
an observation point, whereupon the latent bug newly leads to a                      dation teams for supporting our work over the past decade, and
failure; possibly a mission-critical failure. In this regard, such latent            the many Meta software engineers and testers whose experience,
bugs which currently benefit from failed error propagation are the                   expertise, and engagement have helped shape the ideas presented
software equivalent of unexploded bombs in the code base, they                       here. We would also like to thank the many academics and other
simply await detonation.                                                             researchers with whom we have had the enormous pleasure to
   A Catching JiTTest is ‘just-in-time’ for the pull request that it                 interact over 3+ decades of research work on verification, testing
tests. It might therefore appear that, by definition, it cannot have                 and AI.
a role to play in catching latent bugs, but this is not the case. We
can take an arbitrary section of code of interest, delete it, and then
re-insert it as a new pull request. For example, suppose we pick a
particular method, 𝑚. We delete 𝑚 and subsequently re-insert it
as a new pull request. Suppose a Functionality Catching JiTTest,
FSE Companion ’25, June 23–28, 2025, Trondheim, Norway                                                                                             Harman, O’Hearn, Sengupta


REFERENCES                                                                                 [19] Thierry Titcheu Chekam, Mike Papadakis, Yves Le Traon, and Mark Harman.
 [1] John Ahlgren, Maria Eugenia Berezin, Kinga Bojarczuk, Elena Dulskyte, Inna                 2017. An empirical study on mutation, statement and branch coverage fault
     Dvortsova, Johann George, Natalija Gucevska, Mark Harman, Ralf Laemmel, Erik               revelation that avoids the unreliable clean program assumption. In Proceedings
     Meijer, Silvia Sapora, and Justin Spahr-Summers. 2020. WES: Agent-based User               of the 39th International Conference on Software Engineering, ICSE 2017, Buenos
                                                                                                Aires, Argentina, May 20-28, 2017. 597–608.
     Interaction Simulation on Real Infrastructure (keynote paper). In 8𝑡ℎ Genetic
                                                                                           [20] Maxime Cordy, Renaud Rwemalika, Adriano Franci, Mike Papadakis, and Mark
     improvement workshop (GI at ICSE 2020), Shin Yoo, Justyna Petke, Westley Weimer,
                                                                                                Harman. 2022. FlakiMe: Laboratory-Controlled Test Flakiness Impact Assessment.
     and Bobby R. Bruce (Eds.). ACM, 276–284.
                                                                                                In 44th IEEE/ACM 44th International Conference on Software Engineering, ICSE
 [2] John Ahlgren, Maria Eugenia Berezin, Kinga Bojarczuk, Elena Dulskyte, Inna
                                                                                                2022, Pittsburgh, PA, USA, May 25-27, 2022. ACM, 982–994.
     Dvortsova, Johann George, Natalija Gucevska, Mark Harman, Maria Lomeli, Erik
                                                                                           [21] Richard A. De Millo, Richard J. Lipton, and Alan J. Perlis. 1979. Social Processes
     Meijer, Silvia Sapora, and Justin Spahr-Summers. 2021. Testing Web Enabled
                                                                                                and Proofs of Theorems and Programs. Commun. ACM 22, 5 (May 1979), 271–280.
     Simulation at Scale Using Metamorphic Testing. In International Conference on
                                                                                                An earlier version appeared in ACM Symposium on Principles of Programming
     Software Engineering (ICSE) Software Engineering in Practice (SEIP) track. Virtual.
                                                                                                Languages (POPL) , Los Angeles, California , 1977 pp. 206–214.
 [3] John Ahlgren, Kinga Bojarczuk, Sophia Drossopoulou, Inna Dvortsova, Johann
                                                                                           [22] Edsger W. Dijkstra. 1978. On a Political Pamphlet from the Middle Ages (A
     George, Natalija Gucevska, Mark Harman, Maria Lomeli, Simon Lucas, Erik
                                                                                                response to the paper ‘Social Processes and Proofs of Theorems and Programs’
     Meijer, Steve Omohundro, Rubmary Rojas, Silvia Sapora, Jie M. Zhang, and Norm
                                                                                                by DeMillo, Lipton, and Perlis). ACM SIGSOFT, Software Engineering Notes 3, 2
     Zhou. 2021. Facebook’s Cyber–Cyber and Cyber–Physical Digital Twins (keynote
                                                                                                (1978), 14–17.
     paper). In 25th International Conference on Evaluation and Assessment in Software
                                                                                           [23] Dino Distefano, Manuel Fähndrich, Francesco Logozzo, and Peter W O’Hearn.
     Engineering (EASE 2021). Virtual.
                                                                                                2019. Scaling static analyses at Facebook. Commun. ACM 62, 8 (2019), 62–70.
 [4] Nadia Alshahwan, Arianna Blasi, Kinga Bojarczuk, Andrea Ciancone, Natalija
                                                                                           [24] Rajendra T Dodhiawala, NS Sridharan, Peter Raulefs, and Cynthia Pickering. 1989.
     Gucevska, Mark Harman, Michal Krolikowski, Rubmary Rojas, Dragos Martac,
                                                                                                Real-Time AI Systems: A Definition and An Architecture.. In IJCAI. Citeseer,
     Simon Schellaert, et al. 2024. Enhancing Testing at Meta with Rich-State Simu-
                                                                                                256–264.
     lated Populations. In Proceedings of the 46th International Conference on Software
                                                                                           [25] Sebastian G. Elbaum, Hui Nee Chin, Matthew B. Dwyer, and Matthew Jorde. 2009.
     Engineering: Software Engineering in Practice. 1–12.
                                                                                                Carving and Replaying Differential Unit Test Cases from System Test Cases. IEEE
 [5] Nadia Alshahwan, Jubin Chheda, Anastasia Finegenova, Mark Harman, Alexan-
                                                                                                Transactions on Software Engineering 35, 1 (2009), 29–45.
     dru Marginean, Shubho Sengupta, and Eddy Wang. 2024. Automated unit test
                                                                                           [26] Angela Fan, Beliz Gokkaya, Mitya Lyubarskiy, Mark Harman, Shubho Sengupta,
     improvement using Large Language Models at Meta. In ACM International Con-
                                                                                                Shin Yoo, and Jie Zhang. 2023. Large Language Models for Software Engineering:
     ference on the Foundations of Software Engineering (FSE 2024) (Porto de Galinhas,
                                                                                                Survey and Open Problems. In ICSE Future of Software Engineering (FoSE 2023).
     Brazil, Brazil).
                                                                                           [27] Zhiyu Fan, Haifeng Ruan, Sergey Mechtaev, and Abhik Roychoudhury. 2024.
 [6] Nadia Alshahwan, Xinbo Gao, Mark Harman, Yue Jia, Ke Mao, Alexander Mols,
                                                                                                Oracle-guided Program Selection from Large Language Models. In Proceedings of
     Taijin Tei, and Ilya Zorin. 2018. Deploying Search Based Software Engineering
                                                                                                the 33rd ACM SIGSOFT International Symposium on Software Testing and Analysis.
     with Sapienz at Facebook (keynote paper). In 10𝑡ℎ International Symposium                  628–640.
     on Search Based Software Engineering (SSBSE 2018). Montpellier, France, 3–45.         [28] Jean Claude Fernandez, Claude Jard, Thierry Jéron, and César Viho. 1996. Using
     Springer LNCS 11036.                                                                       on-the-fly verification techniques for the generation of test suites. In Computer
 [7] Nadia Alshahwan, Mark Harman, and Alexandru Marginean. 2023. Software                      Aided Verification: 8th International Conference, CAV’96 New Brunswick, NJ, USA,
     Testing Research Challenges: An Industrial Perspective (keynote paper). In 2023            July 31–August 3, 1996 Proceedings 8. Springer, 348–359.
     IEEE Conference on Software Testing, Verification and Validation (ICST 2023). IEEE,   [29] Christopher Foster, Abhishek Gulati, Mark Harman, Inna Harper, Ke Mao, Jillian
     1–10.                                                                                      Ritchey, Hervé Robert, and Shubho Sengupta. 2025. Mutation-Guided LLM-based
 [8] Nadia Alshahwan, Mark Harman, Alexandru Marginean, Shubho Sengupta, and                    Test Generation at Meta. In 2025 ACM Conference on Foundations of Software
     Eddy Wang. 2024. Assured LLM-Based Software Engineering (keynote paper).                   Engineering (FSE 2025). ACM. Also available as arXiv preprint arXiv:2501.12862.
     In 2𝑛𝑑 . ICSE workshop on Interoperability and Robustness of Neural Software          [30] Phyllis G. Frankl, Stewart N. Weiss, and Cang Hu. 1997. All-Uses vs Mutation Test-
     Engineering (InteNSE) (Lisbon, Portugal).                                                  ing: An Experimental Comparison of Effectiveness. Journal of Systems Software
 [9] Nadia Alshahwan, Mark Harman, Alexandru Marginean, and Eddy Wang. 2024.                    38 (1997), 235–253.
     Observation-based unit test generation at Meta. In Foundations of Software Engi-      [31] Gordon Fraser and Andrea Arcuri. 2011. EvoSuite: automatic test suite generation
     neering (FSE 2024).                                                                        for object-oriented software. In 8𝑡ℎ European Software Engineering Conference
[10] Source Code Analysis and Manipulation Workshop (SCAM). 2001. SCAM Char-                    and the ACM SIGSOFT Symposium on the Foundations of Software Engineering
     ter. https://www.ieee-scam.org/ The cited quotation has remained part of the               (ESEC/FSE ’11). ACM, 416–419.
     workshop’s credo since its foundation in 2001.                                        [32] Patrice Godefroid, Nils Klarlund, and Koushik Sen. 2005. DART: directed auto-
[11] Saswat Anand, Antonia Bertolino, Edmund Burke, Tsong Yueh Chen, John Clark,                mated random testing. In Programming Language Design and Implementation
     Myra B. Cohen, Wolfgang Grieskamp, Mark Harman, Mary Jean Harrold, Jenny                   (PLDI 2005), Vivek Sarkar and Mary W. Hall (Eds.). ACM, 213–223.
     Li, Phil McMinn, and Hong Zhu. 2013. An orchestrated survey of methodologies          [33] Rahul Gopinath, Carlos Jensen, and Alex Groce. 2014. Code coverage for suite
     for automated software test case generation. Journal of Systems and Software 86,           evaluation by developers. In Proceedings of the 36th international conference on
     8 (August 2013), 1978–2001.                                                                software engineering. 72–82.
[12] Kelly Androutsopoulos, David Clark, Haitao Dan, Mark Harman, and Robert               [34] Mark Harman. 2010. Why Source Code Analysis and Manipulation Will Always
     Hierons. 2014. An Analysis of the Relationship between Conditional Entropy and
                                                                                                Be Important (Keynote Paper). In 10𝑡ℎ IEEE International Working Conference on
     Failed Error Propagation in Software Testing. In 36𝑡ℎ International Conference             Source Code Analysis and Manipulation. Timisoara, Romania.
     on Software Engineering (ICSE 2014). Hyderabad, India, 573–583.                       [35] Mark Harman, Yue Jia, and William B. Langdon. 2011. Strong Higher Order
[13] Earl T. Barr, Mark Harman, Phil McMinn, Muzammil Shahbaz, and Shin Yoo.
                                                                                                Mutation-Based Test Data Generation. In 8𝑡ℎ European Software Engineering
     2015. The Oracle Problem in Software Testing: A Survey. IEEE Transactions on
                                                                                                Conference and the ACM SIGSOFT Symposium on the Foundations of Software
     Software Engineering 41, 5 (May 2015), 507–525.
                                                                                                Engineering (ESEC/FSE ’11) (Szeged, Hungary). ACM, New York, NY, USA, 212–
[14] Carolin Brandt and Andy Zaidman. 2022. Developer-centric test amplification:
                                                                                                222.
     The interplay between automatic generation human exploration. Empirical
                                                                                           [36] Mark Harman, Yue Jia, and Yuanyuan Zhang. 2015. Achievements, open problems
     Software Engineering 27, 4 (2022), 96.
[15] Cristian Cadar, Daniel Dunbar, Dawson R Engler, et al. 2008. Klee: unassisted              and challenges for search based software testing (keynote Paper). In 8𝑡ℎ IEEE
     and automatic generation of high-coverage tests for complex systems programs..             International Conference on Software Testing, Verification and Validation (ICST
     In OSDI, Vol. 8. 209–224.                                                                  2015). Graz, Austria.
[16] Cristian Cadar, Patrice Godefroid, Sarfraz Khurshid, Corina S. Păsăreanu, Koushik     [37] Mark Harman and Bryan F. Jones. 2001. Search Based Software Engineering.
     Sen, Nikolai Tillmann, and Willem Visser. 2011. Symbolic execution for software            Information and Software Technology 43, 14 (Dec. 2001), 833–839.
                                                                                           [38] Mark Harman, Afshin Mansouri, and Yuanyuan Zhang. 2012. Search Based
     testing in practice: preliminary assessment. In 33𝑟𝑑 International Conference on
                                                                                                Software Engineering: Trends, Techniques and Applications. Comput. Surveys
     Software Engineering (ICSE’11) (Waikiki, Honolulu, HI, USA). ACM, New York,
                                                                                                45, 1 (November 2012), 11:1–11:61.
     NY, USA, 1066–1071.
                                                                                           [39] Mark Harman and Peter O’Hearn. 2018. From Start-ups to Scale-ups: Opportu-
[17] Cristian Cadar and Koushik Sen. 2013. Symbolic Execution for Software Testing:
                                                                                                nities and Open Problems for Static and Dynamic Program Analysis (keynote
     Three Decades Later. Commun. ACM 56, 2 (Feb. 2013), 82–90.
[18] C. Calcagno, D. Distefano, J. Dubreil, D. Gabi, P. Hooimeijer, M. Luca, P. W.              paper). In 18𝑡ℎ IEEE International Working Conference on Source Code Analysis
     O’Hearn, I. Papakonstantinou, J. Purbrick, and D. Rodriguez. 2015. Moving                  and Manipulation (SCAM 2018). Madrid, Spain, 1–23.
     Fast with Software Verification. In NASA Formal Methods - 7th International           [40] Mark Harman and Laurence Tratt. 2007. Pareto Optimal Search-based Refactor-
     Symposium. 3–11.                                                                           ing at the Design Level. In 9𝑡ℎ annual conference on Genetic and evolutionary
                                                                                                computation (GECCO 2007). ACM Press, London, UK, 1106 – 1113.
Harden and Catch for Just-in-Time Assured LLM-Based Software Testing: Open Research Challenges                        FSE Companion ’25, June 23–28, 2025, Trondheim, Norway


[41] Mark Harman, Xiangjuan Yao, and Yue Jia. 2014. A Study of Equivalent and                [64] Facundo Molina, Alessandra Gorla, and Marcelo d’Amorim. 2024. Test Oracle
     Stubborn Mutation Operators Using Human Analysis of Equivalence. In 36𝑡ℎ                     Automation in the era of LLMs. ACM Transactions on Software Engineering and
     International Conference on Software Engineering (ICSE 2014). Hyderabad, India,              Methodology (2024).
     919–930.                                                                                [65] Morten Mossige, Arnaud Gotlieb, and Hein Meling. 2015. Testing robot controllers
[42] Kazi Amit Hasan, Marcos Macedo, Yuan Tian, Bram Adams, and Steven Ding.                      using constraint programming and continuous integration. Information and
     2023. Understanding the time to first response in GitHub pull requests. In 2023              Software Technology 57 (2015), 169–185.
     IEEE/ACM 20th International Conference on Mining Software Repositories (MSR).           [66] Vijayaraghavan Murali, Chandra Maddila, Imad Ahmad, Michael Bolin, Daniel
     IEEE, 1–11.                                                                                  Cheng, Negar Ghorbani, Renuka Fernandez, Nachiappan Nagappan, and Peter C
[43] Rob Hierons, Kirill Bogdanov, Jonathan Bowen, Rance Cleaveland, John Derrick,                Rigby. 2024. CodeCompose: A Large-Scale Industrial Deployment of AI-assisted
     Jeremy Dick, Marian Gheorghe, Mark Harman, Kalpesh Kapoor, Paul Krause,                      Code Authoring. In Foundations of Software Engineering (FSE 2024). Porto de
     Gerald Luettgen, Tony Simons, Sergiy Vilkomir, Martin Woodward, and Hussein                  Galinhas, Brazil. Earlier version available as arXiv:2305.12050.
     Zedan. 2009. Using Formal Methods to Support Testing. Comput. Surveys 41, 2             [67] Peter W O’Hearn. 2019. Incorrectness logic. Proceedings of the ACM on Program-
     (Feb. 2009). Article 9.                                                                      ming Languages 4, POPL (2019), 1–32.
[44] Charles Anthony Richard Hoare. 1996. How did software get so reliable without           [68] Mike Papadakis, Yue Jia, Mark Harman, and Yves Le Traon. 2015. Trivial Com-
     proof?. In FME ’96: Industrial Benefit and Advances in Formal Methods: Third                 piler Equivalence: A Large Scale Empirical Study of a Simple, Fast and Effective
     International Symposium of Formal Methods Europe (LNCS, 1051). Springer-Verlag,              Equivalent Mutant Detection Technique. In 37𝑡ℎ International Conference on
     1–17.                                                                                        Software Engineering (ICSE 2015). Florence, Italy, 936–946.
[45] Charles Anthony Richard Hoare. 1996. How did software get so reliable without           [69] Justyna Petke, Saemundur O. Haraldsson, Mark Harman, William B. Langdon,
     proof?. In IEEE International Conference on Software Engineering (ICSE’96). IEEE             David R. White, and John R. Woodward. 2018. Genetic Improvement of Software:
     Computer Society Press, Los Alamitos, California, USA. Keynote talk and                      a Comprehensive Survey. IEEE Transactions on Evolutionary Computation 22, 3
     extended abstract.                                                                           (June 2018), 415–432.
[46] Steven CH Hoi, Doyen Sahoo, Jing Lu, and Peilin Zhao. 2021. Online learning: A          [70] Juan Altmayer Pizzorno and Emery D Berger. 2024. Coverup: Coverage-guided
     comprehensive survey. Neurocomputing 459 (2021), 249–289.                                    LLM-based test generation. arXiv preprint arXiv:2403.16218 (2024).
[47] Soneya Binta Hossain and Matthew Dwyer. 2024. TOGLL: Correct and strong                 [71] Outi Räihä. 2010. A survey on Search–Based Software Design. Computer Science
     test oracle generation with LLMs. arXiv preprint arXiv:2405.03786 (2024).                    Review 4, 4 (2010), 203–249.
[48] Eugene Ie, Chih-wei Hsu, Martin Mladenov, Vihan Jain, Sanmit Narvekar, Jing             [72] Aurora Ramirez, Jose Raul Romero, and Christopher L Simons. 2018. A systematic
     Wang, Rui Wu, and Craig Boutilier. 2019. RecSim: A Configurable Simulation                   review of interaction in search-based software engineering. IEEE Transactions on
     Platform for Recommender Systems. arXiv e-prints, Article arXiv:1909.04847                   Software Engineering 45, 8 (2018), 760–781.
     (Sep 2019). arXiv:1909.04847 [cs.LG]                                                    [73] Aurora Ramirez, José Raúl Romero, and Sebastian Ventura. 2019. A survey of
[49] Laura Inozemtseva and Reid Holmes. 2014. Coverage is not strongly correlated                 many-objective optimisation in search-based software engineering. Journal of
     with test suite effectiveness. In Proceedings of the 36th international conference on        Systems and Software 149 (2019), 382–395.
     software engineering. 435–445.                                                          [74] David Schuler and Andreas Zeller. 2009. Javalanche: efficient mutation testing for
[50] Kush Jain, Gabriel Synnaeve, and Baptiste Rozière. 2024. TestGenEval: A real                 Java. In 7𝑡ℎ joint meeting of the European Software Engineering Conference and the
     world unit test generation and test completion benchmark. arXiv preprint                     ACM SIGSOFT International Symposium on Foundations of Software Engineering
     arXiv:2410.00752 (2024).                                                                     (ESEC/FSE 2009). 297–298.
[51] Yue Jia and Mark Harman. 2011. An Analysis and Survey of the Development of             [75] Koushik Sen, Darko Marinov, and Gul Agha. 2005. CUTE: a concolic unit testing
     Mutation Testing. IEEE Transactions on Software Engineering 37, 5 (September–                engine for C. In 10𝑡ℎ European Software Engineering Conference and 13th ACM
     October 2011), 649 – 678.                                                                    International Symposium on Foundations of Software Engineering (ESEC/FSE ’05),
[52] René Just, Darioush Jalali, Laura Inozemtseva, Michael D. Ernst, Reid Holmes,                Michel Wermelinger and Harald Gall (Eds.). ACM, 263–272.
     and Gordon Fraser. 2014. Are Mutants a Valid Substitute for Real Faults in Software     [76] Christopher L. Simons, Ian C. Parmee, and Rhys Gwynllyw. 2010. Interactive, Evo-
     Testing? Technical Report UW-CSE-14-02-02. University of Washington.                         lutionary Search in Upstream Object-Oriented Class Design. IEEE Transactions
[53] Donald E. Knuth. 1984. Literate Programming. Comput. J. 27, 2 (1984), 97–111.                on Software Engineering 36, 6 (2010), 798–816.
[54] Michael Konstantinou, Renzo Degiovanni, and Mike Papadakis. 2024. Do LLMs               [77] John Steven, Pravir Chandra, Bob Fleck, and Andy Podgurski. 2000. jRapture: A
     generate test oracles that capture the actual or the expected program behaviour?             capture/replay tool for observation-based testing. In Proceedings of the 2000 ACM
     arXiv preprint arXiv:2410.21136 (2024).                                                      SIGSOFT international symposium on Software Testing and Analysis (ISSTA 2000).
[55] Hermann Kopetz and Wilfried Steiner. 2022. Real-Time Communication. In Real-                 158–167.
     time systems: Design principles for distributed embedded applications. Springer,        [78] Deepika Tiwari, Martin Monperrus, and Benoit Baudry. 2023. Mimicking Pro-
     177–200.                                                                                     duction Behavior with Generated Mocks. arXiv:2208.01321 [cs.SE]
[56] Kiran Lakhotia, Mark Harman, and Hamilton Gross. 2013. AUSTIN: An Open                  [79] Shreshth Tuli, Kinga Bojarczuk, Natalija Gucevska, Mark Harman, Xiao-Yu Wang,
     Source Tool for Search Based Software Testing of C Programs. Journal of Infor-               and Graham Wright. 2023. Simulation-Driven Automated End-to-End Test and
     mation and Software Technology 55, 1 (January 2013), 112–125.                                Oracle Inference. In 45th IEEE/ACM International Conference on Software Engi-
[57] Qingzhou Luo, Farah Hariri, Lamyaa Eloussi, and Darko Marinov. 2014. An                      neering: Software Engineering in Practice, SEIP@ICSE 2023, Melbourne, Australia,
     empirical analysis of flaky tests. In 22𝑛𝑑 International Symposium on Foundations            May 14-20, 2023. IEEE, 122–133.
     of Software Engineering (FSE 2014), Shing-Chi Cheung, Alessandro Orso, and              [80] Lars van Hijfte and Ana Oprescu. 2021. Mutantbench: an equivalent mutant
     Margaret-Anne Storey (Eds.). ACM, Hong Kong, China, 643–653.                                 problem comparison framework. In 2021 IEEE International Conference on Software
[58] Lech Madeyski, Wojciech Orzeszyna, Richard Torkar, and Mariusz Jozala. 2013.                 Testing, Verification and Validation Workshops (ICSTW). IEEE, 7–12.
     Overcoming the equivalent mutant problem: A systematic literature review and a          [81] Margus Veanes, Colin Campbell, Wolfram Schulte, Pushmeet Kohli, N Tillmann,
     comparative experiment of second order mutation. IEEE Transactions on Software               and W Grieskamp. 2005. On-the-fly testing of reactive systems. Submitted for
     Engineering 40, 1 (2013), 23–42.                                                             publication (2005).
[59] Valentin J. M. Manès, HyungSeok Han, Choongwoo Han, Sang Kil Cha, Manuel                [82] Junjie Wang, Yuchao Huang, Chunyang Chen, Zhe Liu, Song Wang, and Qing
     Egele, Edward J. Schwartz, and Maverick Woo. 2018. The Art, Science, and                     Wang. 2023. Software testing with large language model: Survey, landscape, and
     Engineering of Fuzzing: A Survey. CoRR abs/1812.00140 (2018). arXiv:1812.00140               vision. arXiv preprint arXiv:2307.07221 (2023).
[60] Ke Mao, Mark Harman, and Yue Jia. 2016. Sapienz: Multi-objective Automated              [83] Zejun Wang, Kaibo Liu, Ge Li, and Zhi Jin. 2024. HITS: High-coverage LLM-based
     Testing for Android Applications. In International Symposium on Software Testing             Unit Test Generation via Method Slicing. In Proceedings of the 39th IEEE/ACM
     and Analysis (ISSTA 2016). 94–105.                                                           International Conference on Automated Software Engineering. 1258–1268.
[61] Ke Mao, Timotej Kapus, Lambros Petrou, Ákos Hajdu, Matteo Marescotti, Andreas           [84] Michal Zalewski. Accessed March 27th 2025. American fuzzy lop. http://lcamtuf.
     Löscher, Mark Harman, and Dino Distefano. 2022. FAUSTA: Scaling Dynamic                      coredump.cx/afl/
     Analysis with Traffic Generation at WhatsApp. In 15th IEEE Conference on Soft-          [85] Shuyin Zhao. 2023. GitHub Copilot now has a better AI model and new capa-
     ware Testing, Verification and Validation, ICST 2022, Valencia, Spain, April 4-14,           bilities. https://github.blog/2023-02-14-github-copilot-now-has-a-better-ai-
     2022. IEEE, 267–278.                                                                         model-and-new-capabilities/
[62] James McGill. 2025. Time to First Review. https://docs.velocity.codeclimate.            [86] Yuxiang Zhu and Minxue Pan. 2019. Automatic code summarization: A systematic
     com/en/articles/2913584-time-to-first-review                                                 literature review. arXiv preprint arXiv:1909.04352 (2019).
[63] Phil McMinn. 2004. Search-based Software Test Data Generation: A Survey.
     Software Testing, Verification and Reliability 14, 2 (June 2004), 105–156.
