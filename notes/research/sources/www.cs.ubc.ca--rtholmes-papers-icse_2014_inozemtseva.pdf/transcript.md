---
url: https://www.cs.ubc.ca/~rtholmes/papers/icse_2014_inozemtseva.pdf
title: Coverage Is Not Strongly Correlated
fetched: 2026-06-27
raw: raw.pdf
transport: curl
capture_status: ok
---

Coverage Is Not Strongly Correlated
                                   with Test Suite Effectiveness
                                                     Laura Inozemtseva and Reid Holmes
                                                                 School of Computer Science
                                                                    University of Waterloo
                                                                   Waterloo, ON, Canada
                                                       {lminozem,rtholmes}@uwaterloo.ca


ABSTRACT                                                                            1.    INTRODUCTION
The coverage of a test suite is often used as a proxy for                              Testing is an important part of producing high quality
its ability to detect faults. However, previous studies that                        software, but its eﬀectiveness depends on the quality of the
investigated the correlation between code coverage and test                         test suite: some suites are better at detecting faults than
suite eﬀectiveness have failed to reach a consensus about the                       others. Naturally, developers want their test suites to be good
nature and strength of the relationship between these test                          at exposing faults, necessitating a method for measuring the
suite characteristics. Moreover, many of the studies were                           fault detection eﬀectiveness of a test suite. Testing textbooks
done with small or synthetic programs, making it unclear                            often recommend coverage as one of the metrics that can
whether their results generalize to larger programs, and some                       be used for this purpose (e.g., [29, 34]). This is intuitively
of the studies did not account for the confounding inﬂuence                         appealing, since it is clear that a test suite cannot ﬁnd bugs
of test suite size. In addition, most of the studies were done                      in code it never executes; it is also supported by studies that
with adequate suites, which are are rare in practice, so the                        have found a relationship between code coverage and fault
results may not generalize to typical test suites.                                  detection eﬀectiveness [3, 6, 14–17, 24, 31, 39].
   We have extended these studies by evaluating the relation-                          Unfortunately, these studies do not agree on the strength
ship between test suite size, coverage, and eﬀectiveness for                        of the relationship between these test suite characteristics.
large Java programs. Our study is the largest to date in the                        In addition, three issues with the studies make it diﬃcult to
literature: we generated 31,000 test suites for ﬁve systems                         generalize their results. First, some of the studies did not
consisting of up to 724,000 lines of source code. We measured                       control for the size of the suite. Since coverage is increased
the statement coverage, decision coverage, and modiﬁed con-                         by adding code to existing test cases or by adding new test
dition coverage of these suites and used mutation testing to                        cases to the suite, the coverage of a test suite is correlated
evaluate their fault detection eﬀectiveness.                                        with its size. It is therefore not clear that coverage is related
   We found that there is a low to moderate correlation                             to eﬀectiveness independently of the number of test cases in
between coverage and eﬀectiveness when the number of test                           the suite. Second, all but one of the studies used small or
cases in the suite is controlled for. In addition, we found that                    synthetic programs, making it unclear that their results hold
stronger forms of coverage do not provide greater insight                           for the large programs typical of industry. Third, many of the
into the eﬀectiveness of the suite. Our results suggest that                        studies only compared adequate suites; that is, suites that
coverage, while useful for identifying under-tested parts of a                      fully satisﬁed a particular coverage criterion. Since adequate
program, should not be used as a quality target because it is                       test suites are rare in practice, the results of these studies
not a good indicator of test suite eﬀectiveness.                                    may not generalize to more realistic test suites.
                                                                                       This paper presents a new study of the relationship between
Categories and Subject Descriptors                                                  test suite size, coverage and eﬀectiveness. We answer the
D.2.5 [Software Engineering]: Testing and Debugging;                                following research questions for large Java programs:
D.2.8 [Software Engineering]: Metrics—product metrics                                 Research Question 1. Is the eﬀectiveness of a test suite
                                                                                    correlated with the number of test cases in the suite?
General Terms
Measurement                                                                           Research Question 2. Is the eﬀectiveness of a test suite
                                                                                    correlated with its statement coverage, decision coverage
Keywords                                                                            and/or modiﬁed condition coverage when the number of test
Coverage, test suite eﬀectiveness, test suite quality                               cases in the suite is ignored?
                                                                                      Research Question 3. Is the eﬀectiveness of a test suite
                                                                                    correlated with its statement coverage, decision coverage
Permission to make digital or hard copies of all or part of this work for           and/or modiﬁed condition coverage when the number of test
personal or classroom use is granted without fee provided that copies are           cases in the suite is held constant?
not made or distributed for proﬁt or commercial advantage and that copies
bear this notice and the full citation on the ﬁrst page. To copy otherwise, to        The paper makes the following contributions:
republish, to post on servers or to redistribute to lists, requires prior speciﬁc        • A comprehensive survey of previous studies that inves-
permission and/or a fee.
ICSE ’14, May 31–June 7, 2014, Hyderabad, India                                            tigated the relationship between coverage and eﬀective-
Copyright 14 ACM 978-1-4503-2756-5/14/05 ...$15.00.                                        ness (Section 2 and accompanying online material).
                                 Table 1: Summary of the ﬁndings from previous studies.
     Citation    Languages       Largest Program Coverage Types                     Findings
      [15, 16]   Pascal          78 SLOC                All-use, decision    All-use related to eﬀectiveness independently of
                                                                             size; decision is not; relationship is highly non-
                                                                             linear
        [17]     Fortran         78 SLOC                All-use, mutation    Eﬀectiveness improves with coverage but not until
                 Pascal                                                      coverage reaches 80%; even then increase is small
        [14]     C               5,905 SLOC             All-use, decision    Eﬀectiveness is correlated with both all-use and
                                                                             decision coverage; increase is small until high levels
                                                                             of coverage are reached
        [39]     C               <2,310 SLOC            Block                Eﬀectiveness is more highly correlated with block
                                                                             coverage than with size
        [24]     C               512 SLOC               All-use, decision    Eﬀectiveness is correlated with both all-use and de-
                                                                             cision coverage; eﬀectiveness increases more rapidly
                                                                             at high levels of coverage
        [6]      C               4,512 SLOC             Block, c-use,        Eﬀectiveness is moderately correlated with all four
                                                        decision, p-use      coverage types; magnitude of the correlation de-
                                                                             pends on the nature of the tests
        [3]      C               5,000 SLOC             Block, c-use,        Eﬀectiveness is correlated with all four coverage
                                                        decision, p-use      types; eﬀectiveness rises steadily with coverage
        [31]     C               5,680 SLOC             Block, c-use,        Eﬀectiveness is correlated with all four coverage
                 C++                                    decision, p-use      types but the correlations are not always strong
      [19, 37]   C               72,490 SLOC            AIMP, DBB,           Eﬀectiveness correlated with coverage; eﬀective-
                 Java                                   decision, IMP,       ness correlated with size for large projects
                                                        PCC, statement
        [5]      C               4,000 SLOC             Block, c-use,        None of the four coverage types are related to
                                                        decision, p-use      eﬀectiveness independently of size
        [20]     Java            O(100, 000)            Block, decision,     Eﬀectiveness correlated with coverage across many
                                 SLOC                   path, statement      projects; inﬂuence of project size unclear

      • Empirical evidence demonstrating that there is a low          relationship between the coverage and the eﬀectiveness of
        to moderate correlation between coverage and eﬀective-        a test suite, ten of which used the general procedure just
        ness when suite size is controlled for and that the type      described. Eight of them found that at least one type of cov-
        of coverage used has little eﬀect on the strength of the      erage has some correlation with eﬀectiveness independently
        relationship (Section 4).                                     of size; however, not all studies found a strong correlation,
      • A discussion of the implications of these results for de-     and most found that the relationship was highly non-linear.
        velopers, researchers and standards bodies (Section 5).       In addition, some found that the relationship only appeared
                                                                      at very high levels of coverage. For brevity, the older stud-
                                                                      ies from Table 1 are described more fully in accompanying
2.     RELATED WORK                                                   materials1 . In the remainder of this section, we discuss the
  Most of the previous studies that investigated the link             three most recent studies.
between test suite coverage and test suite eﬀectiveness used             At the time of writing, no other study considered any
the following general procedure:                                      subject program larger than 5,905 SLOC2 . However, a recent
     1. Created faulty versions of one or more programs by            study by Gligoric et al. [19] and a subsequent master’s the-
        manually seeding faults, reintroducing previously ﬁxed        sis [37] partially addressed this issue by studying two large
        faults, or using a mutation tool.                             Java programs (JFreeChart and Joda Time) and two large C
                                                                      programs (SQLITE and YAFFS2) in addition to a number
     2. Created a large number of test suites by selecting from
                                                                      of small programs. The authors created test suites by sam-
        a pool of available test cases, either randomly or accord-
                                                                      pling from the pool of test cases for each program. For the
        ing to some algorithm, until the suite reached either a
                                                                      large programs, these test cases were manually written by
        pre-speciﬁed size or a pre-speciﬁed coverage level.
                                                                      developers; for the small programs, these test cases were auto-
     3. Measured the coverage of each suite in one or more            matically generated using various tools. Suites were created
        ways, if suite size was ﬁxed; measured the suite’s size
        if its coverage was ﬁxed.
                                                                      1
     4. Determined the eﬀectiveness of each suite as the frac-          http://linozemtseva.com/research/2014/icse/
        tion of faulty versions of the program that were detected     coverage/
                                                                      2
        by the suite.                                                   In this paper, source lines of code (SLOC) refers to exe-
                                                                      cutable lines of code, while lines of code (LOC) includes
     Table 1 summarizes twelve studies that considered the            whitespace and comments.
in two ways. First, the authors speciﬁed a coverage level and      3.1   Terminology
selected tests until it was met; next, the authors speciﬁed a        Before describing the methodology in detail, we precisely
suite size and selected tests until it was met. They measured      deﬁne three terms that will be used throughout the paper.
a number of coverage types: statement coverage, decision
                                                                      • Test case: one test in a suite of tests. A test case
coverage, and more exotic measurements based on equivalent
                                                                        executes as a unit; it is either executed or not executed.
classes of covered statements (dynamic basic block coverage),
                                                                        In the JUnit testing framework, each method that starts
program paths (intra-method and acyclic intra-method path
                                                                        with the word test (JUnit 3) or that is annotated with
coverage), and predicate states (predicate complete cover-
                                                                        @Test (JUnit 4) is a test case. For this reason, we will
age). They evaluated the eﬀectiveness of each suite using
                                                                        use the terms test method and test case interchangeably.
mutation testing. They found that the Kendall τ correla-
tion (see Section 4.2) between coverage and mutation score            • Test suite: a collection of test cases.
ranged from 0.452 to 0.757 for the various coverage types             • Master suite: the whole test suite that was written
and suite types when the size of the suite was not considered.          by the developers of a subject program. For example,
When they tried to predict the mutation score using suite               the master suite for Apache POI contains 1,415 test
size alone, they found high correlations (between 0.585 and             cases (test methods). The test suites that we create
0.958) for the four large programs with manually written                and evaluate are strict subsets of the master suite.
test suites but fairly low correlations for the small programs
with artiﬁcially generated test suites. This suggests that the     3.2   Subject Programs
correlation between coverage and eﬀectiveness in real systems         We selected ﬁve subjects from a variety of application
is largely due to the correlation between coverage and size; it    domains. The ﬁrst, Apache POI [4], is an open source API
also suggests that results from automatically generated and        for manipulating Microsoft documents. The second, Closure
manually generated suites do not generalize to each other.         Compiler [7], is an open source JavaScript optimizing com-
   A study by Gopinath et al. [20] accepted to the same con-       piler. The third, HSQLDB [23], is an open source relational
ference as the current paper did not use the aforementioned        database management system. The fourth, JFreeChart [25],
general procedure. The authors instead measured coverage           is an open source library for producing charts. The ﬁfth,
and test suite eﬀectiveness for a large number of open-source      Joda Time [26], is an open source replacement for the Java
Java programs and computed a correlation across all pro-           Date and Time classes.
grams. Speciﬁcally, they measured statement, block, decision          We used a number of criteria to select these projects.
and path coverage and used mutation testing to measure             First, to help ensure the novelty and generalizability of our
eﬀectiveness. The authors measured these values for approx-        study, we required that the projects be reasonably large (on
imately 200 developer-generated test suites – the number           the order of 100,000 SLOC), written in Java, and actively
varies by measurement – then generated a suite for each            developed. We also required that the projects have a fairly
project with the Randoop tool [36] and repeated the mea-           large number of test methods (on the order of 1,000) so that
surements. The authors found that coverage is correlated           we would be able to generate reasonably sized random test
with eﬀectiveness across projects for all coverage types and       suites. Finally, we required that the projects use Ant as
for both developer-generated and automatically-generated           a build system and JUnit as a test harness, allowing us to
suites, though the correlation was stronger for developer-         automate data collection.
written suites. The authors found that including test suite           The salient characteristics of our programs are summarized
size in their regression model did not improve the results;        in Table 2. Program size was measured with SLOCCount [38].
however, since coverage was already included in the model,         Rows seven through ten provide information related to mu-
it is not clear whether this is an accurate ﬁnding or a result     tation testing and will be explained in Section 3.3.
of multicollinearity3 .
   As the above discussion shows, it is still not clear how        3.3   Generating Faulty Programs
test suite size, coverage and eﬀectiveness are related. Most
                                                                      We used the open source tool PIT [35] to generate faulty
studies conclude that eﬀectiveness is related to coverage, but
                                                                   versions of our programs. To describe PIT’s operation, we
there is little agreement about the strength and nature of
                                                                   must ﬁrst give a brief description of mutation testing.
the relationship.
                                                                      A mutant is a new version of a program that is created
                                                                   by making a small syntactic change to the original program.
3.     METHODOLOGY                                                 For example, a mutant could be created by modifying a
  To answer our research questions, we followed the general        constant, negating a branch condition, or removing a method
procedure outlined in Section 2. This required us to select:       call. The resulting mutant may produce the same output as
     1. A set of subject programs (Section 3.2);                   the original program, in which case it is called an equivalent
     2. A method of generating faulty versions of the programs     mutant. For example, if the equality test in the code snippet
        (Section 3.3);                                             in Figure 1 were changed to if (index >= 10), the new
     3. A method of creating test suites (Section 3.4);            program would be an equivalent mutant.
     4. Coverage metrics (Section 3.5); and                           Mutation testing tools such as PIT generate a large number
     5. An eﬀectiveness metric (Section 3.6).                      of mutants and run the program’s test suite on each one.
                                                                   If the test suite fails when it is run on a given mutant, we
We then measured the coverage and eﬀectiveness of the suites       say that the suite kills that mutant. A test suite’s mutant
to evaluate the relationship between these characteristics.        coverage is then the fraction of non-equivalent mutants
3
  The amount of variation ‘explained’ by a variable will be        that it kills. Equivalent mutants are excluded because they
less if it is correlated with a variable already included in the   cannot, by deﬁnition, be detected by a unit test.
model than it would be otherwise.                                     If a mutant is not killed by a test suite, manual inspec-
                             Table 2: Salient characteristics of our subject programs.
                      Property           Apache POI Closure HSQLDB JFreeChart                                 Joda Time
           Total Java SLOC                          283,845          724,089    178,018         125,659          80,462
           Test SLOC                                 68,932           93,528     18,425          44,297          51,444
           Number of test methods                     1,415            7,947        628           1,764           3,857
           Statement coverage (%)                        67               76         27              54              91
           Decision coverage (%)                         60               77         17              45              82
           MC coverage (%)                               49               67          9              27              70
           Number of mutants                         27,565           30,779      50,302         29,699           9,552
           Number of detected mutants                17,935           27,325      50,125         23,585           8,483
           Number of equivalent mutants               9,630            3,454         177          6,114           1,069
           Equivalent mutants (%)                        35               11           0.4           21              11

int index = 0;
while (true) {
                                                                         3.4   Generating Test Suites
    index++;                                                               For each subject program, we used Java’s reﬂection API to
    if (index == 10) {                                                  identify all of the test methods in the program’s master suite.
        break;                                                          We then generated new test suites of ﬁxed size by randomly
    }                                                                   selecting a subset of these methods without replacement.
}                                                                       More concretely, we created a JUnit suite by repeatedly
                                                                        using the TestSuite.addTest(Test t) method. Each suite
Figure 1: An example of how an equivalent mutant                        was created as a JUnit suite so that the necessary set-up and
can be generated. Changing the operator == to >=                        tear-down code was run for each test method. Given this
will result in a mutant that cannot be detected by                      procedure for creating suites, in this paper the size of our
an automated test case.                                                 random suites should always be understood as the number of
                                                                        test methods they contain, i.e., the number of times addTest
tion is required to determine if it is equivalent or if it was          was called.
simply missed by the suite4 . This is a time-consuming and                 We made 1,000 suites of each of the following sizes: 3
error-prone process, so studies that compare subsets of a               methods, 10 methods, 30 methods, 100 methods, and so on,
test suite to the master suite often use a diﬀerent approach:           up to the largest number following this pattern that was less
they assume that any mutant that cannot be detected by                  than the total number of test methods. This resulted in a
the master suite is equivalent. While this technique tends              total of 31,000 test suites across the ﬁve subject systems.
to overestimate the number of equivalent mutants, it is com-            Comparing a large number of suites from the same project
monly applied because it allows the study of much larger                allows us to control for size; it also allows us to apply our
programs.                                                               results to the common research practice of comparing test
   Although the mutants generated by PIT simulate real                  suites generated for the same subject program using diﬀerent
faults, it is not self-evident that a suite’s ability to kill mu-       test generation methodologies.
tants is a valid measurement of its ability to detect real faults.
However, several previous and current studies support the                3.5   Measuring Coverage
use of this measurement [2, 3, 10, 27]. Previous work has also             We used the open source tool CodeCover [8] to measure
shown that if a test suite detects a large number of simple             three types of coverage: statement, decision, and modiﬁed
faults, caused by a single incorrect line of source code, it            condition coverage. Statement coverage refers to the fraction
will detect a large number of harder, multi-line faults [28, 32].       of the executable statements in the program that are run
This implies that if a test suite can kill a large proportion of        by the test suite. It is relatively easy to satisfy, easy to
mutants, it can also detect a large proportion of the more              understand and can be measured quickly, making it popular
diﬃcult faults in the software. The literature thus suggests            with developers. However, it is one of the weaker forms of
that the mutant detection rate of a suite is a fairly good              coverage, since executing a line does not necessarily reveal
measurement of its fault detection ability. We will return to           an error in that line.
this issue in Sections 6 and 7.                                            Decision coverage refers to the fraction of decisions (i.e.,
   We can now describe the remaining rows of Table 2. The               branches) in the program that are executed by its test suite.
seventh row shows how many mutants PIT generated for each               Decision coverage is somewhat harder to satisfy and measure
project. The eighth row shows how many of those mutants                 than statement coverage.
could be detected by the suite. The ninth row shows how                    Modiﬁed condition coverage (MCC) is the most diﬃcult
many of those mutants could not be detected by the entire               of these three to satisfy. For a test suite to be modiﬁed
test suite and were therefore assumed to be equivalent (i.e.,           condition adequate, i.e., to have 100% modiﬁed condition
row 7 is the sum of rows 8 and 9). The last row gives the               coverage, the suite must include 2n test cases for every deci-
equivalent mutants as a percentage of the total.                        sion with n conditions5 in it [22]. This form of coverage is not
                                                                        commonly used in practice; however, it is very similar to mod-
                                                                         5
                                                                          A condition is a boolean expression that cannot be de-
4
 Manual inspection is required because automatically deter-              composed into a simpler boolean expression. Decisions are
mining whether a mutant is equivalent is undecidable [33].               composed of conditions and one or more boolean operators.
iﬁed condition/decision coverage (MC/DC), which is widely          measurement of 1 and a raw eﬀectiveness measurement of
used in the avionics industry. Speciﬁcally, Federal Aviation       1, since we decided that any mutants it did not kill are
Administration standard DO-178B states that the most criti-        equivalent.
cal software in the aircraft must be tested with a suite that is
modiﬁed condition/decision coverage adequate [22]. MC/DC           4.     RESULTS
is therefore one of the most stringent forms of coverage that        In this section, we quantitatively answer the three research
is widely and regularly used in practice. Measuring modiﬁed        questions posed in Section 1. As Section 3 explained, we
condition coverage provides insight into whether stronger          collected the data to answer these questions by generating
coverage types such as MCC and MC/DC provide practical             test suites of ﬁxed size via random sampling; measuring their
beneﬁts that outweigh the extra cost associated with writing       statement, decision and MCC coverage with CodeCover; and
enough tests to satisfy them.                                      measuring their eﬀectiveness with the mutation testing tool
   We did not measure any type of dataﬂow coverage, since          PIT.
very few tools for Java can measure these types of coverage.
One exception is Coverlipse [9], which can measure all-use         4.1      Is Size Correlated With Effectiveness?
coverage but can only be used as an Eclipse plugin. To the            Research Question 1 asked if the eﬀectiveness of a test suite
best of our knowledge, there are no open source coverage tools     is inﬂuenced by the number of test methods it contains. This
for Java that can measure other data ﬂow coverage criteria         research question provides a “sanity check” that supports the
or that can be used from the command line. Since developers        use of the eﬀectiveness metric. Figure 2 shows some of the
use the tools they have, they are unlikely to use dataﬂow          data we collected to answer this question. In each subﬁgure,
coverage metrics. Using the measurements that developers           the x axis indicates suite size on a logarithmic scale while the
use, whether due to tool availability or legal requirements,       y axis shows the range of normalized eﬀectiveness values we
means that our results will more accurately reﬂect current         computed. The red line on each plot was ﬁt to the data with
development practice. However, we plan to explore dataﬂow          R’s lm function6 . The adjusted r2 value for each regression
coverage in future work to determine if developers would           line is shown in the bottom right corner of each plot. These
beneﬁt from using these coverage types instead.                    values range from 0.26 to 0.97, implying that the correlation
                                                                   coeﬃcient r ranges from 0.51 to 0.98. This indicates that
3.6    Measuring Effectiveness                                     there is a moderate to very high correlation between normal-
   We used two eﬀectiveness measurements in this study:            ized eﬀectiveness and size for these projects7 . The results for
the raw eﬀectiveness measurement and the normalized eﬀec-          the non-normalized eﬀectiveness measurement are similar,
tiveness measurement. The raw kill score is the number of          with the r2 values ranging from 0.69 to 0.99, implying a high
mutants a test suite detected divided by the total number of       to very high correlation between non-normalized eﬀective-
non-equivalent mutants that were generated for the subject         ness and size. The ﬁgure for this measurement can be found
program under test. The normalized eﬀectiveness measure-           online8 .
ment is the number of mutants a test suite detected divided
                                                                           Answer 1. Our results suggest that, for large Java
by the number of non-equivalent mutants it covers. A test
                                                                        programs, there is a moderate to very high correlation
suite covers a mutant if the mutant was made by altering a
                                                                        between the eﬀectiveness of a test suite and the number
line of code that is executed by the test suite, implying that
                                                                        of test methods it contains.
the test suite can potentially detect the mutant.
   We included the normalized eﬀectiveness measurement in
order to compare test suites on a more even footing. Suppose       4.2      Is Coverage Correlated With Effectiveness
we are comparing suite A, with 50% coverage, to suite B, with               When Size Is Ignored?
60% coverage. Suite B will almost certainly have a higher
                                                                      Research Question 2 asked if the eﬀectiveness of a test suite
raw eﬀectiveness measurement, since it covers more code and
                                                                   is correlated with the coverage of the suite when we ignore
will therefore almost certainly kill more mutants. However,
                                                                   the inﬂuence of suite size. Tables 3 and 4 show the Kendall τ
if suite A kills 80% of the mutants that it covers, while suite
                                                                   correlation coeﬃcients we computed to answer this question;
B kills only 70% of the mutants that it covers, suite A is
                                                                   all coeﬃcients are signiﬁcant at the 99.9% level9 . Table 3
in some sense a better suite. The normalized eﬀectiveness
                                                                   6
measurement captures this diﬀerence. Note that it is possible        Size and the logarithm of size were used as the inputs.
                                                                   7
for the normalized eﬀectiveness measurement to drop when             Here we use the Guildford scale [21] for verbal description,
a new test case is added to the suite if the test case covers a    in which correlations with absolute value less than 0.4 are
lot of code but kills few mutants.                                 described as “low”, 0.4 to 0.7 as “moderate”, 0.7 to 0.9 as
   It may be helpful to think of the normalized eﬀectiveness       “high”, and over 0.9 as “very high”.
                                                                   8
measurement as a measure of depth: how thoroughly does the           http://linozemtseva.com/research/2014/icse/
                                                                   coverage/
test suite exercise the code that it runs? The raw eﬀectiveness    9
                                                                     Kendall’s τ is similar to the more common Pearson coef-
measurement is a measure of breadth: how much code does            ﬁcient but does not assume that the variables are linearly
the suite exercise?                                                related or that they are normally distributed. Rather, it
   Note that the number of non-equivalent mutants covered          measures how well an arbitrary monotonic function could ﬁt
by a suite is the maximum number of mutants the suite could        the data. A high correlation therefore means that we can
possibly detect, so the normalized eﬀectiveness measurement        predict the rank order of the suites’ eﬀectiveness values given
ranges from 0 to 1. The raw eﬀectiveness measurement,              the rank order of their coverage values, which in practice
                                                                   is nearly as useful as predicting an absolute eﬀectiveness
in general, does not reach 1, since most suites kill a small       score. We used it instead of the Pearson coeﬃcient to avoid
percentage of the non-equivalent mutants. However, note            introducing unnecessary assumptions about the distribution
that the full test suite has both a normalized eﬀectiveness        of the data.
                                              Apache POI                                   Closure                                 HSQLDB
                              1.00
                                     ●
                                     ●
                                     ●                                                                           ●
                                     ●
                                     ●                          ●
                                     ●
                                     ●
                                     ●
                                     ●                                                                    ●
                                                                                                          ●
                                     ●
                                     ●                    ●
                                                                                                                        ●
                                                    ●
                                                    ●
                                                    ●                                                                   ●
                                                          ●                                                             ●

                              0.75
                                         ●     ●          ●                                          ●
                                         ●     ●                                                     ●
                                                                                                     ●
                                         ●
                                         ●     ●
                                               ●                                                                              ●
                                         ●     ●
                                               ●                                                                              ●
                                                                                                                              ●
                                                                                                                              ●    ●
                                                                                                                                   ●
                                         ●
                                         ●                                                                                    ●
                                                                                                                              ●    ●
                                                                                                                                   ●   ●
                                                                                                                                       ●
                                                                                                                                       ●
                                         ●                                                    ●
                                                                                              ●      ●
                                                                                                     ●                        ●
                                                                                                                              ●    ●
                                                                                                                                   ●   ●
                                                                                                                                       ●
                                                    ●                                  ●      ●
                                                                                              ●                               ●    ●   ●
                                                                                                                                       ●
                                                                                                                              ●    ●
                                                                                                                                   ●   ●
                                                                                                                                       ●
                                                                                       ●
                                                                                       ●
                                                                                  ●    ●
                                                                                       ●
                                                                                       ●
                                                                                                                              ●
                                                                                  ●
                                                                                  ●           ●                                    ●
                                               ●
                                               ●
                                                                                                                        ●



                              0.50
                                                                                       ●
                                                                                       ●
                                                                                       ●
                                                                                       ●
                                         ●
                                         ●                                             ●
                                         ●
                                         ●                                             ●
                                         ●
                                         ●                                        ●
                                                                                  ●
                                         ●                                        ●
                                                                                  ●
                                                                                  ●
                                                                                  ●
                                         ●                                        ●
                                                                                  ●
                                         ●
                                         ●
                                                                              ●
                                     ●
                                     ●                                        ●
                                     ●
                                     ●                                        ●
                                     ●
                                     ●                                        ●
                                                                              ●
                                     ●
                                     ●
                                     ●                                        ●

                              0.25
                                     ●                                        ●
                                     ●
                                     ●                                        ●
                                     ●
                                     ●
                                     ●                                        ●
                                     ●
                                     ●                                        ●
                                     ●
                                     ●
                                     ●
                                     ●
                                     ●
              Effectiveness




                                                        R^2 = 0.78                                R^2 = 0.97                               R^2 = 0.26
                              0.00

                                              JFreeChart                               Joda Time
                              1.00                                                                               ●
                                                                                                                 ●
                                                                ●
                                                          ●
                                                          ●
                                               ●                              ●                           ●
                                               ●
                                               ●                                                     ●    ●
                                               ●
                                               ●    ●
                                                    ●     ●
                                                                                              ●
                                                                                              ●
                                                                                       ●      ●

                              0.75                  ●
                                                    ●                             ●
                                                                                  ●
                                                                                       ●
                                                                                       ●
                                                                                       ●
                                                                                       ●
                                                                                                     ●



                                                                                              ●
                                               ●
                                               ●




                              0.50
                                                                                       ●
                                                                                       ●
                                                                                       ●
                                                                                       ●
                                                                                       ●




                                                                                  ●
                                                                                  ●

                                                                                  ●



                              0.25


                                                        R^2 = 0.55                                R^2 = 0.89
                              0.00
                                                                1000
                                                                       3000




                                                                                                          1000
                                                                                                                 3000
                                                    100
                                                          300




                                                                                              100
                                                                                                    300
                                         10
                                               30




                                                                                  10
                                                                                       30
                                     3




                                                                              3




                                                                                            Size
Figure 2: Normalized eﬀectiveness scores plotted against size for all subjects. Each box represents the 1000
suites of a given size that were created from a given master suite.

gives the correlation between the diﬀerent coverage types
and the normalized eﬀectiveness measurement. Table 4 gives                                   Table 3: The Kendall τ correlation between nor-
the correlation between the diﬀerent coverage types and the                                  malized eﬀectiveness and diﬀerent types of coverage
non-normalized eﬀectiveness measurement. For all projects                                    when suite size is ignored. All entries are signiﬁcant
but HSQLDB, we see a moderate to very high correlation                                       at the 99.9% level.
between coverage and eﬀectiveness when size is not taken                                        Project    Statement Decision Mod. Cond.
into account. HSQLDB is an interesting exception: when the                                        Apache POI             0.75               0.76         0.77
eﬀectiveness measurement is normalized by the number of                                           Closure                0.83               0.83         0.84
covered mutants, there is a low negative correlation between                                      HSQLDB                −0.35              −0.35        −0.35
coverage and eﬀectiveness. This means that the suites with                                        JFreeChart             0.50               0.53         0.53
higher coverage kill fewer mutants per unit of coverage; in                                       Joda Time              0.80               0.80         0.80
other words, the suites with higher coverage contain test
cases that run a lot of code but do not kill many mutants
in that code. Of course, since the suites kill more mutants                                  Table 4: The Kendall τ correlation between non-
in total as they grow, there is a positive correlation between                               normalized eﬀectiveness and diﬀerent types of cov-
coverage and non-normalized eﬀectiveness for HSQLDB.                                         erage when suite size is ignored. All entries are sig-
                                                                                             niﬁcant at the 99.9% level.
     Answer 2. Our results suggest that, for many large                                         Project    Statement Decision Mod. Cond.
  Java programs, there is a moderate to high correlation
  between the eﬀectiveness and the coverage of a test suite                                       Apache POI                0.94             0.94        0.94
  when the inﬂuence of suite size is ignored. Research                                            Closure                   0.95             0.95        0.95
  Question 3 explores whether this correlation is caused                                          HSQLDB                    0.81             0.80        0.79
  by the larger size of the suites with higher coverage.                                          JFreeChart                0.91             0.95        0.92
                                                                                                  Joda Time                 0.85             0.85        0.85

4.3   Is Coverage Correlated With Effectiveness                                              the results we obtained for one project and one suite size.
      When Size Is Fixed?                                                                    The project name is given at the top of each column, while
  Research Question 3 asked if the eﬀectiveness of a test                                    the suite size is given at the right of each row. Diﬀerent
suite is correlated with its coverage when the number of                                     coverage types are diﬀerentiated by colour. The bottom row
test cases in the suite is controlled for. Figure 3 shows the                                is a margin plot that shows the results for all sizes, while the
data we collected to answer this question. Each panel shows                                  rightmost column is a margin plot that shows the results for
                                Apache POI                       Closure                     HSQLDB                  JFreeChart                  Joda Time                          (all)
                       1.00
                       0.75
                       0.50




                                                                                                                                                                                                       3
                       0.25
                       0.00
                       1.00
                       0.75




                                                                                                                                                                                                       10
                       0.50
                       0.25
                       0.00
                       1.00
                       0.75




                                                                                                                                                                                                       30
                       0.50
                       0.25
                       0.00
                       1.00
                       0.75




                                                                                                                                                                                                       100
       Effectiveness




                       0.50
                       0.25
                       0.00
                       1.00
                       0.75




                                                                                                                                                                                                       300
                       0.50
                       0.25
                       0.00
                       1.00
                       0.75




                                                                                                                                                                                                       1000
                       0.50                                                                     N/A
                       0.25
                       0.00
                       1.00
                       0.75




                                                                                                                                                                                                       3000
                       0.50            N/A                                                     N/A                         N/A
                       0.25
                       0.00
                       1.00
                       0.75




                                                                                                                                                                                                       (all)
                       0.50
                       0.25
                       0.00
                              0.00
                                     0.25
                                            0.50
                                                   0.75
                                                          1.00
                                                          0.00
                                                                 0.25
                                                                        0.50
                                                                               0.75
                                                                                      1.00
                                                                                      0.00
                                                                                             0.25
                                                                                                    0.50
                                                                                                           0.75
                                                                                                                  1.00
                                                                                                                  0.00
                                                                                                                         0.25
                                                                                                                                0.50
                                                                                                                                       0.75
                                                                                                                                              1.00
                                                                                                                                              0.00
                                                                                                                                                     0.25
                                                                                                                                                            0.50
                                                                                                                                                                   0.75
                                                                                                                                                                          1.00
                                                                                                                                                                          0.00
                                                                                                                                                                                 0.25
                                                                                                                                                                                        0.50
                                                                                                                                                                                               0.75
                                                                                                                                                                                                      1.00
                                                                                                           Coverage

                              Coverage Type ● Decision coverage ● Modified condition coverage ● Statement coverage

Figure 3: Normalized eﬀectiveness scores (left axis) plotted against coverage (bottom axis) for all subjects.
Rows show the results for one suite size; columns show the results for one project. N/A indicates that the
project did not have enough test cases to ﬁll in that frame.

all projects. The ﬁgure shows the results for the normalized                                                        age once size was controlled for while the non-normalized
eﬀectiveness measurement; the non-normalized eﬀectiveness                                                           eﬀectiveness measurements had moderate correlations with
measurements tend to be small and diﬃcult to see at this size.                                                      coverage once size was controlled for.
The ﬁgure for the non-normalized eﬀectiveness measurement                                                             That said, the results varied by project. Joda Time was
can be found online with the other supplementary material.                                                          at one extreme: the correlation between coverage and ef-
   We computed the Kendall τ correlation coeﬃcient between                                                          fectiveness ranged from 0.80 to 0.85 when suite size was
eﬀectiveness and coverage for each project, each suite size,                                                        ignored, but dropped to essentially zero when suite size was
each coverage type, and both eﬀectiveness measures. Since                                                           controlled for. The same eﬀect was seen for Closure when
this resulted in a great deal of data, we summarize the results                                                     the normalized eﬀectiveness measurement was used.
here; the full dataset can be found on the same website as                                                            Apache POI fell at the other extreme. For this project,
the ﬁgures.                                                                                                         the correlation between coverage and the non-normalized
   Our results were mixed. Controlling for suite size always                                                        eﬀectiveness measurement was 0.94 when suite size was ig-
lowered the correlation between coverage and eﬀectiveness.                                                          nored, but dropped to a range of 0.46 to 0.85 when suite size
However, the magnitude of the change depended on the ef-                                                            was controlled for. While this is in some cases a large drop,
fectiveness measurement used. In general, the normalized                                                            a correlation in this range can provide useful information
eﬀectiveness measurements had low correlations with cover-                                                          about the quality of a test suite.
   A very interesting result is that, in general, the coverage                               1
type used did not have a strong impact on the results. This
is true even though the eﬀectiveness scores (y values) for each
suite are the same for all three coverage types (x values).                                 0.8

To clarify this, consider Figure 4. The ﬁgure shows two
hypothetical graphs of eﬀectiveness against coverage. In
                                                                                            0.6




                                                                            Effectiveness
the top graph, coverage type 1 is not strongly correlated
with eﬀectiveness. In the bottom graph, coverage type 2 is
strongly correlated with eﬀectiveness even though the y-value                               0.4       ●
of each point has not changed (e.g., the triangle is at y = 0.8
in both graphs). We do not see this diﬀerence between
statement, decision, and MCC coverage, suggesting that the                                  0.2

diﬀerent types of coverage are measuring the same thing.
We can conﬁrm this intuition by measuring the correlation
                                                                                             0
between diﬀerent coverage types for each suite (Table 5).




                                                                                                      0.2




                                                                                                              0.4




                                                                                                                        0.6




                                                                                                                              0.8
                                                                                                  0




                                                                                                                                    1
Given these high correlations, and given that the shape of                                                  Coverage type 1
the point clouds are similar for all three coverage measures
(see Figure 3), we can conclude that the coverage type used
has little eﬀect on the relationship between coverage and                                    1

eﬀectiveness in this study.
                                                                                            0.8
Table 5: The Kendall τ and Pearson correlations be-
tween diﬀerent types of coverage for all suites from
all projects.                                                                               0.6




                                                                            Effectiveness
          Coverage Types   Tau Pearson
            Statement/Decision      0.92      0.99                                          0.4              ●
            Decision/MCC            0.91      0.98
            Statement/MCC           0.92      0.97
                                                                                            0.2



        Answer 3. Our results suggest that, for large Java
                                                                                             0
     programs, the correlation between coverage and eﬀec-
                                                                                                      0.2




                                                                                                              0.4




                                                                                                                        0.6




                                                                                                                              0.8
                                                                                                  0




                                                                                                                                    1
     tiveness drops when suite size is controlled for. After                                                Coverage type 2
     this drop, the correlation typically ranges from low to
     moderate, meaning it is not generally safe to assume
     that eﬀectiveness is correlated with coverage. The corre-    Figure 4: Hypothetical graphs of eﬀectiveness
     lation is stronger when the non-normalized eﬀectiveness      against two coverage types for four test suites. The
     measurement is used. Additionally, the type of cov-          top graph shows a coverage type that is not corre-
     erage used had little inﬂuence on the strength of the        lated with eﬀectiveness; the bottom graph shows a
     relationship.                                                coverage type that is correlated with eﬀectiveness.


5.     DISCUSSION                                                 clouds corresponding to the three coverage types are similar
  The goal of this work was to determine if a test suite’s        in shape and size. This, in combination with the high cor-
coverage is correlated with its fault detection eﬀectiveness      relation between diﬀerent coverage measurements, suggests
when suite size is controlled for. We found that there is         that stronger coverage types provide little extra information
typically a moderate to high correlation between coverage         about the quality of the suite.
and eﬀectiveness when suite size is ignored, and that this           Our ﬁndings have implications for developers, researchers,
drops to a low to moderate correlation when size is con-          and standards bodies. Developers may wish to use this
trolled. This result suggests that coverage alone is not a        information to guide their use of coverage. While coverage
good predictor of test suite eﬀectiveness; in many cases, the     measures are useful for identifying under-tested parts of a
apparent relationship is largely due to the fact that high        program, and low coverage may indicate that a test suite is
coverage suites contain more test cases. The results for Joda     inadequate, high coverage does not indicate that a test suite
Time and Closure, in particular, demonstrate that it is not       is eﬀective. This means that using a ﬁxed coverage value as
safe in general to assume that coverage is correlated with        a quality target is unlikely to produce an eﬀective test suite.
eﬀectiveness. Interestingly, the suites for Joda Time and         While members of the testing community have previously
Closure are the largest and most comprehensive of the ﬁve         made this point [13, 30], it has been diﬃcult to evaluate their
suites we studied, which might indicate that the correlation      suggestions due to a lack of studies that considered systems of
becomes weaker as the suite improves.                             the scale that we investigated. Additionally, it may be in the
  In addition, we found that the type of coverage measured        developer’s best interest to use simpler coverage measures.
had little impact on the correlation between coverage and         These measures provide a similar amount of information
eﬀectiveness. This is reinforced by the shape of the point        about the suite’s eﬀectiveness but introduce less measurement
clouds in Figure 3: for any one project and suite size, the       overhead.
  Researchers may wish to use this information to guide            to compute each τ . We found that ties rarely occurred: for
their tool-building. In particular, test generation techniques     the worst calculation, 4.6% of the comparisons resulted in a
often attempt to maximize the coverage of the resulting suite;     tie, but for most calculations this percentage was smaller by
our results suggest that this may not be the best approach.        several orders of magnitude. Since there were so few ties, we
  Finally, our results are pertinent to standards bodies that      have assumed that they had a negligible eﬀect on the normal
set requirements for software testing. The FAA standard            distribution.
DO-178B, mentioned earlier in this paper, requires the use of         Another threat to internal validity stems from the possi-
MC/DC adequate suites to ensure the quality of the resulting       bility of duplicate test suites: our results might be skewed if
software; however, our results suggest that this requirement       two or more suites contain the same subset of test methods.
may increase expenses without necessarily increasing quality.      Fortunately, we can evaluate this threat using the informa-
  Of course, developers still want to measure the quality          tion we collected about ties: since duplicate suites would
of their test suites, meaning they need a metric that does         naturally have identical coverage and eﬀectiveness scores,
correlate with fault detection ability. While this is still an     the number of tied comparisons provides an upper bound
open problem, we currently feel that mutation score may be         on how many identical suites were compared. Since the
a good substitute for coverage in this context [27].               number of ties was so low, the number of duplicate suites
                                                                   must be similarly low, and so we have ignored the small skew
6.    THREATS TO VALIDITY                                          they may have introduced to avoid increasing the memory
                                                                   requirements of our study unnecessarily.
  In this section, we discuss the threats to the construct
                                                                      Since we have studied correlations, we cannot make any
validity, internal validity, and external validity of our study.
                                                                   claims about the direction of causality.
6.1    Construct Validity
   In our study we measured the size, coverage and eﬀective-       6.3    External Validity
ness of random test suites. Size and coverage are straight-           There are six main threats to the external validity of our
forward to measure, but eﬀectiveness is more nebulous, as          study. First, previous work suggests that the relationship
we are attempting to predict the fault-detection ability of a      between size, coverage and eﬀectiveness depends on the dif-
suite that has never been used in practice. As we described        ﬁculty of detecting faults in the program [3]. Furthermore,
in Section 3.3, previous and current work suggests that a          some of the previous work was done with hand-seeded faults,
suite’s ability to kill mutants is a fairly good measurement       which have been shown to be harder to detect than both
of its ability to detect real faults [2, 3, 10, 27]. This sug-     mutants and real faults [2]. While this does not aﬀect our
gests that, in the absence of equivalent mutants, this metric      results, it does make it harder to compare them with those
has high construct validity. Unfortunately, our treatment          of earlier studies.
of equivalent mutants introduces a threat to the validity of          Second, some of the previous studies found that a rela-
this measurement. Recall that we assumed that any mutant           tionship between coverage and eﬀectiveness did not appear
that could not be detected by the program’s entire test suite      until very high coverage levels were reached [14,17,24]. Since
is equivalent. This means that we classiﬁed up to 35% of           the coverage of our generated suites rarely reached very high
the generated mutants as equivalent (see the ﬁnal row of           values, it is possible that we missed the existence of such
Table 2). In theory, these mutants are a random subset of          a relationship. That said, it is not clear that such a rela-
the entire set of mutants, so ignoring them should not aﬀect       tionship would be useful in practice. It is very diﬃcult to
our results. However, this may not be true. For example, if        reach extremely high levels of coverage, so a relationship that
the developers frequently test for oﬀ-by-one errors, mutants       does not appear until 90% coverage is reached is functionally
that simulate this error will be detected more often and will      equivalent to no relationship at all for most developers.
be less likely to be classiﬁed as equivalent.                         Third, in object-oriented systems, most faults are usu-
                                                                   ally found in just a few of the system’s components [12].
6.2    Internal Validity                                           This means that the relationship between size, coverage and
   Our conclusions about the relationship between size, cov-       eﬀectiveness may vary by class within the system. It is there-
erage and eﬀectiveness depend on our calculations of the           fore possible that coverage is correlated with eﬀectiveness
Kendall τ correlation coeﬃcient. This introduces a threat to       in classes with speciﬁc characteristics, such as high churn.
the internal validity of the study. Kendall’s original formula     However, our conclusions still hold for the common practice
for τ assumes that there are no tied ranks in the data; that       of measuring the coverage of a program’s entire test suite.
is, if the data were sorted, no two rows could be exchanged           Fourth, there may be other features of a program or a suite
without destroying the sorted order. When ties do exist,           that aﬀect the relationship between coverage and eﬀective-
two issues arise. First, since the original formula does not       ness. For example, previous work suggests that the size of a
handle ties, a modiﬁed one must be used. We used the ver-          class can aﬀect the validity of object-oriented metrics [11].
sion proposed by Adler [1]. Second, ties make it diﬃcult to        While we controlled for the size of each test suite in this
compute the statistical signiﬁcance of the correlation coef-       study, we did not control for the size of the class that each
ﬁcient. It it possible to show that, in the absence of ties,       test method came from.
τ is normally distributed, meaning we can use Z-scores to             Fifth, as discussed in Section 3.2, our subjects had to
evaluate signiﬁcance in the usual way. However, when ties          meet certain inclusion criteria. This means that they are
are present, the distribution of τ changes in a way that de-       fairly similar, so our results may not generalize to programs
pends on the number and nature of the ties. This can result        that do not meet these criteria. We attempted to mitigate
in a non-normal distribution [18]. To determine the impact         this threat by selecting programs from diﬀerent application
of ties on our calculations, we counted both the number of         domains, thereby ensuring a certain amount of variety in the
ties that occurred and the total number of comparisons done        subjects. Unfortunately, it was diﬃcult to ﬁnd acceptable
subjects; in particular, the requirement that the subjects         [4] Apache POI. http://poi.apache.org.
have 1,000 test cases proved to be very diﬃcult to satisfy. In     [5] L. Briand and D. Pfahl. Using simulation for assessing
practice, it seems that most open source projects do not have          the real impact of test coverage on defect coverage. In
comprehensive test suites. This is supported by Gopinath et            Proc. of the Int’l Symposium on Software Reliability
al.’s study [20], where only 729 of the 1,254 open source Java         Engineering, 1999.
projects they initially considered, or 58%, had test suites at     [6] X. Cai and M. R. Lyu. The eﬀect of code coverage on
all, much less comprehensive suites.                                   fault detection under diﬀerent testing proﬁles. In Proc.
   Finally, while our subjects were considerably larger than           of the Int’l Workshop on Advances in Model-Based
the programs used in previous studies, they are still not large        Testing, 2005.
by industrial standards. Additionally, all of the projects         [7] Closure Compiler.
were open source, so our results may not generalize to closed          https://code.google.com/p/closure-compiler/.
source systems.                                                    [8] CodeCover. http://codecover.org/.
7.    FUTURE WORK                                                  [9] Coverlipse. http://coverlipse.sourceforge.net/.
   Our next step is to conﬁrm our ﬁndings using real faults       [10] M. Daran and P. Thévenod-Fosse. Software error
to eliminate this threat to validity. We will also explore             analysis: a real case study involving real faults and
dataﬂow coverage to determine if these coverage types are              mutations. In Proc. of the Int’l Symposium on Software
correlated with eﬀectiveness.                                          Testing and Analysis, 1996.
   It may also be helpful to perform a longitudinal study that    [11] K. El Emam, S. Benlarbi, N. Goel, and S. N. Rai. The
considers how the coverage and eﬀectiveness of a program’s             confounding eﬀect of class size on the validity of
test suite change over time. By cross-referencing coverage             object-oriented metrics. IEEE Transactions on Soft.
information with bug reports, it might be possible to isolate          Eng., 27(7), 2001.
those bugs that were covered by the test suite but were           [12] N. E. Fenton and N. Ohlsson. Quantitative analysis of
not immediately detected by it. Examining these bugs may               faults and failures in a complex software system. IEEE
provide insight into which bugs are the most diﬃcult to                Transactions on Soft. Eng., 26(8), 2000.
detect and how we can improve our chances of detecting            [13] M. Fowler. Test coverage. http:
them.                                                                  //martinfowler.com/bliki/TestCoverage.html,
                                                                       2012.
8.    CONCLUSION                                                  [14] P. G. Frankl and O. Iakounenko. Further empirical
  In this paper, we studied the relationship between the               studies of test eﬀectiveness. In Proc. of the Int’l
number of methods in a program’s test suite, the suite’s               Symposium on Foundations of Soft. Eng., 1998.
statement, decision, and modiﬁed condition coverage, and the      [15] P. G. Frankl and S. N. Weiss. An experimental
suite’s mutant eﬀectiveness measurement, both normalized               comparison of the eﬀectiveness of the all-uses and
and non-normalized. From the ﬁve large Java programs we                all-edges adequacy criteria. In Proc. of the Symposium
studied, we drew the following conclusions:                            on Testing, Analysis, and Veriﬁcation, 1991.
     • In general, there is a low to moderate correlation be-     [16] P. G. Frankl and S. N. Weiss. An experimental
       tween the coverage of a test suite and its eﬀectiveness         comparison of the eﬀectiveness of branch testing and
       when its size is controlled for.                                data ﬂow testing. IEEE Transactions on Soft. Eng.,
     • The strength of the relationship varies between software        19(8), 1993.
       systems; it is therefore not generally safe to assume      [17] P. G. Frankl, S. N. Weiss, and C. Hu. All-uses vs
       that eﬀectiveness is strongly correlated with coverage.         mutation testing: an experimental comparison of
     • The type of coverage used had little impact on the              eﬀectiveness. Journal of Systems and Software, 38(3),
       strength of the correlation.                                    1997.
                                                                  [18] J. D. Gibbons. Nonparametric Measures of Association.
  These results imply that high levels of coverage do not
                                                                       Sage Publications, 1993.
indicate that a test suite is eﬀective. Consequently, using a
ﬁxed coverage value as a quality target is unlikely to produce    [19] M. Gligoric, A. Groce, C. Zhang, R. Sharma, M. A.
an eﬀective test suite. In addition, complex coverage mea-             Alipour, and D. Marinov. Comparing non-adequate test
surements may not provide enough additional information                suites using coverage criteria. In Proc. of the Int’l
about the suite to justify the higher cost of measuring and            Symp. on Soft. Testing and Analysis, 2013.
satisfying them.                                                  [20] R. Gopinath, C. Jenson, and A. Groce. Code coverage
                                                                       for suite evaluation by developers. In Proc. of the Int’l
                                                                       Conf. on Soft. Eng., 2014.
9.    REFERENCES                                                  [21] J. P. Guilford. Fundamental Statistics in Psychology
 [1] L. M. Adler. A modiﬁcation of Kendall’s tau for the               and Education. McGraw-Hill, 1942.
     case of arbitrary ties in both rankings. Journal of the
                                                                  [22] K. Hayhurst, D. Veerhusen, J. Chilenski, and
     American Statistical Association, 52(277), 1957.                  L. Rierson. A practical tutorial on modiﬁed
 [2] J. H. Andrews, L. C. Briand, and Y. Labiche. Is                   condition/decision coverage. Technical report, NASA
     mutation an appropriate tool for testing experiments?             Langley Research Center, 2001.
     In Proc. of the Int’l Conf. on Soft. Eng., 2005.             [23] HSQLDB. http://hsqldb.org.
 [3] J. H. Andrews, L. C. Briand, Y. Labiche, and A. S.           [24] M. Hutchins, H. Foster, T. Goradia, and T. Ostrand.
     Namin. Using mutation analysis for assessing and                  Experiments of the eﬀectiveness of dataﬂow- and
     comparing testing coverage criteria. IEEE Transactions
                                                                       controlﬂow-based test adequacy criteria. In Proc. of the
     on Soft. Eng., 32(8), 2006.
     Int’l Conf. on Soft. Eng., 1994.                            [32] A. J. Oﬀutt. Investigations of the software testing
[25] JFreeChart. http://jfree.org/jfreechart.                         coupling eﬀect. ACM Transactions on Soft. Eng. and
[26] Joda Time. http://joda-time.sourceforge.net.                     Methodology, 1(1), 1992.
[27] R. Just, D. Jalali, L. Inozemtseva, M. D. Ernst,            [33] A. J. Oﬀutt and J. Pan. Detecting equivalent mutants
     R. Holmes, and G. Fraser. Are mutants a valid                    and the feasible path problem. In Proc. of the Conf. on
     substitute for real faults in software testing? Technical        Computer Assurance, 1996.
     Report UW-CSE-14-02-02, University of Washington,           [34] W. Perry. Eﬀective Methods for Software Testing.
     March 2014.                                                      Wiley Publishing, 2006.
[28] K. Kapoor. Formal analysis of coupling hypothesis for       [35] PIT. http://pitest.org/.
     logical faults. Innovations in Systems and Soft. Eng.,      [36] Randoop. https://code.google.com/p/randoop/.
     2(2), 2006.                                                 [37] R. Sharma. Guidelines for coverage-based comparisons
[29] E. Kit. Software Testing in the Real World: Improving            of non-adequate test suites. Master’s thesis, University
     the Process. ACM Press, 1995.                                    of Illinois at Urbana-Champaign, 2013.
[30] B. Marick. How to misuse code coverage. http://www.         [38] SLOCCount. http://dwheeler.com/sloccount.
     exampler.com/testing-com/writings/coverage.pdf,             [39] W. E. Wong, J. R. Horgan, S. London, and A. P.
     1997.                                                            Mathur. Eﬀect of test set size and block coverage on
[31] A. S. Namin and J. H. Andrews. The inﬂuence of size              the fault detection eﬀectiveness. In Proc. of the Int’l
     and coverage on test suite eﬀectiveness. In Proc. of the         Symposium on Software Reliability Engineering, 1994.
     Int’l Symposium on Software Testing and Analysis,
     2009.
