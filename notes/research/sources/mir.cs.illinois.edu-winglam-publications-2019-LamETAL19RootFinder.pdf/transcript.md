---
url: https://mir.cs.illinois.edu/winglam/publications/2019/LamETAL19RootFinder.pdf
title: Root Causing Flaky Tests in a Large-Scale Industrial Setting
fetched: 2026-06-27
raw: raw.pdf
transport: curl
capture_status: ok
---

Root Causing Flaky Tests in a Large-Scale Industrial Setting
                        Wing Lam                                                 Patrice Godefroid                                            Suman Nath
                University of Illinois at                                       Microsoft Corporation                                   Microsoft Corporation
                 Urbana-Champaign                                                Redmond, WA, USA                                        Redmond, WA, USA
                   Urbana, IL, USA                                               pg@microsoft.com                                      sumann@microsoft.com
                winglam2@illinois.edu

                                                 Anirudh Santhiar                                         Suresh Thummalapenta
                                               Microsoft Corporation                                         Microsoft Corporation
                                                Redmond, WA, USA                                              Redmond, WA, USA
                                              ansanthi@microsoft.com                                       suthumma@microsoft.com
ABSTRACT                                                                                              KEYWORDS
In today’s agile world, developers often rely on continuous integra-                                  flaky tests, debugging, regression testing
tion pipelines to help build and validate their changes by executing
                                                                                                      ACM Reference Format:
tests in an efficient manner. One of the significant factors that
                                                                                                      Wing Lam, Patrice Godefroid, Suman Nath, Anirudh Santhiar, and Suresh
hinder developers’ productivity is flaky tests—tests that may pass                                    Thummalapenta. 2019. Root Causing Flaky Tests in a Large-Scale Industrial
and fail with the same version of code. Since flaky test failures are                                 Setting. In Proceedings of the 28th ACM SIGSOFT International Symposium
not deterministically reproducible, developers often have to spend                                    on Software Testing and Analysis (ISSTA ’19), July 15–19, 2019, Beijing, China.
hours only to discover that the occasional failures have nothing                                      ACM, New York, NY, USA, 11 pages. https://doi.org/10.1145/3293882.3330570
to do with their changes. However, ignoring failures of flaky tests
can be dangerous, since those failures may represent real faults
                                                                                                      1    INTRODUCTION
in the production code. Furthermore, identifying the root cause of
flakiness is tedious and cumbersome, since they are often a con-                                      Both the industry and the open-source community have embraced
sequence of unexpected and non-deterministic behavior due to                                          the Continuous Integration (CI) model of software development and
various factors, such as concurrency and external dependencies.                                       releases [26, 45]. In this model, every check-in is validated through
   As developers in a large-scale industrial setting, we first describe                               an automated pipeline, perhaps running as a service in the cloud,
our experience with flaky tests by conducting a study on them. Our                                    that fetches source code from a version controlled repository, builds
results show that although the number of distinct flaky tests may                                     source code, and runs tests against the built code. These tests must
be low, the percentage of failing builds due to flaky tests can be                                    all pass in order for the developer to integrate changes with the
substantial. To reduce the burden of flaky tests on developers, we                                    master branch. Thus, tests play a central role in ensuring that the
describe our end-to-end framework that helps identify flaky tests                                     changes do not introduce regressions.
and understand their root causes. Our framework instruments flaky                                         To ensure that developers deliver new features at a high velocity,
tests and all relevant code to log various runtime properties, and                                    tests should run quickly and reliably without imposing undue load
then uses a preliminary tool, called RootFinder, to find differences                                  on underlying resources such as build machines. On the other hand,
in the logs of passing and failing runs. Using our framework, we                                      it is also critical that no regressions are introduced into the existing
collect and publicize a dataset of real-world, anonymized execution                                   code. That is, in an ideal world, test failures would reliably signal
logs of flaky tests. By sharing the findings from our study, our                                      issues with the developer’s changes and every test failure would
framework and tool, and a dataset of logs, we hope to encourage                                       warrant investigation. Unfortunately, the reality of CI pipelines
more research on this important problem.                                                              today is that tests may pass and fail with the same version of source
                                                                                                      code and the same configuration. These tests are commonly referred
                                                                                                      to as flaky tests [19, 34]. In a recent keynote [37], Micco observed
CCS CONCEPTS
                                                                                                      that 1.5% of all test runs in Google’s CI pipeline are flaky, and almost
• Software and its engineering → Software testing and de-                                             16% of 4.2 million individual tests fail independently of changes in
bugging.                                                                                              code or tests. Similarly, we find a non-negligible fraction of tests
                                                                                                      to be flaky; over a one-month period monitoring of five software
                                                                                                      projects, we observed that 4.6% of all individual test cases are flaky.
Permission to make digital or hard copies of all or part of this work for personal or
classroom use is granted without fee provided that copies are not made or distributed                 Given their prevalence, there have even been calls to adopt the
for profit or commercial advantage and that copies bear this notice and the full citation             position that every test is potentially a flaky test [24].
on the first page. Copyrights for components of this work owned by others than the                        The presence of flaky tests imposes a significant burden on de-
author(s) must be honored. Abstracting with credit is permitted. To copy otherwise, or
republish, to post on servers or to redistribute to lists, requires prior specific permission         velopers using CI pipelines. In a survey conducted on 58 Microsoft
and/or a fee. Request permissions from permissions@acm.org.                                           developers, we find that they considered flaky tests to be the sec-
ISSTA ’19, July 15–19, 2019, Beijing, China                                                           ond most important reason, out of 10 reasons, for slowing down
© 2019 Copyright held by the owner/author(s). Publication rights licensed to ACM.
ACM ISBN 978-1-4503-6224-5/19/07. . . $15.00                                                          software deployments. A further detailed survey using 18 of the
https://doi.org/10.1145/3293882.3330570                                                               developers showed that they value debugging and fixing existing




                                                                                                101
ISSTA ’19, July 15–19, 2019, Beijing, China                                                  W. Lam, P. Godefroid, S. Nath, A. Santhiar, and S. Thummalapenta


flaky tests as the third most important course of action for Microsoft         runtime behavior of flaky tests can be an effective way to help de-
to take regarding flaky tests. These debugging and fixing efforts              velopers root cause flakiness in tests. More details about the lessons
are often complicated by the fact that the test failures may only              we learned are presented in Section 5.
occur intermittently, and are sometimes reproducible only on the                  Overall, this paper consists of two parts. The first part contains
CI pipeline but not on local machines. When we re-run flaky tests              a motivational study and a dataset of flaky tests. The second part
locally 100 times, we find that 86% of them are only flaky in the              contains a preliminary tool and framework to help identify and
CI pipeline. This result is not surprising since reproducing flaky             debug flaky tests. The paper makes the following contributions:
behavior entails triggering a particular execution among many                  Study: A large-scale study of flaky tests in an industrial setting. Our
possible non-deterministic executions for a given flaky test. Non-             quantitative results demonstrate that flaky tests are prevalent, and
determinism can arise from the non-availability of external I/O                that although the number of distinct flaky tests is comparatively
resources, such as network or disk, or from the order of thread and            low, the number of validation runs that could fail due to flaky
event scheduling. Prior work [34, 42, 51] uncovering these factors             tests is high, emphasizing the problematic nature of flaky tests to
has examined in detail the extent to which these factors contribute            developers. Along with the quantitative results, we also provide
towards flakiness, and developers often find it too difficult and              qualitative insights with in-depth examples that demonstrate the
expensive to identify the root cause of flakiness.                             root causes of flaky tests.
   In most cases of flaky test failures, developers often resort to            Dataset: A collection of publicly available real-world, anonymized
rerunning the tests after spending considerable effort debugging               execution logs of flaky tests. We hope that the data will spur the
the failures [36, 37]. In some cases, developers may request that the          development of new research techniques to root cause flaky tests
flaky test should be rerun up to n times with the hope that at least           and evaluate the effectiveness of tools that can help investigate
one of the runs passes. For large repositories with multiple flaky             flaky tests. These logs are publicly available online [8].
tests that may fail independently, this process could add substantial          Tool: We develop and make publicly available a preliminary tool [8],
time to the developer feedback loop, and can also result in scarcity           called RootFinder, that analyzes the logs of passing and failing
of resources shared across teams. For example, Google ends up                  executions of the same test to suggest method calls that could be
using ≈ 2-16% of its testing budget just to rerun flaky tests [37].            responsible for the flakiness.
Ignoring flaky test failures can be dangerous since a rerun test that          Framework: An end-to-end framework developed within Microsoft
passes might hide a real bug in the production code. One study [43]            that uses RootFinder to root cause flaky tests. This framework can
found that when developers ignored flaky test failures during a                be easily replicated for other companies and open-source projects.
build, the deployed build experienced many more crashes than
builds that did not contain any flaky test failures. Another recent
study [49] also found developers treating the signals from tests as            2   OUR EXPERIENCE WITH FLAKY TESTS
unreliable, sometimes choosing to simply ignore test failures.                 Microsoft makes a CI pipeline available to its developers as a mod-
   Debugging flaky test failures is a difficult and time-consuming ac-         ern build and test service framework on the cloud, called CloudBuild.
tivity, yet important to ensure high-quality production code. There-           CloudBuild is an incremental and distributed system for building
fore, there is a pressing need to develop automated scalable tech-             code and executing unit tests, similar to other engineering systems
niques to help developers debug flaky tests. This work takes the first         such as Bazel [2] and Buck [3].
step in this direction. More specifically, we study the prevalence of             When a developer sends a build request to CloudBuild with a
flaky tests in an industrial setting using production data obtained            change, CloudBuild constructs a dependency graph of all modules
within Microsoft. We then describe an end-to-end framework that                in the project and identifies the modules that are impacted by the
identifies the flaky tests during regular test executions and helps            given change. CloudBuild executes unit tests only in those impacted
understand the root causes of the flakiness. This framework can be             modules, and skips the remaining modules’ unit tests, since none
replicated for other companies and open-source projects.                       of their dependencies were changed. Note that, within a module,
   Our experience with flaky tests includes dynamically instrument-            CloudBuild always executes all tests in the same order. Also, Cloud-
ing flaky test executions. Even though we focus here on unit test              Build executes unit tests as soon as their dependencies are ready,
executions, we observe 335k methods calls, 5 threads, and 55418                rather than waiting for the entire build to finish. This is a major
objects on average per test run. Future efforts on flaky tests should          advantage that helps reduce the overall build time since tests are
hopefully be able to handle programs of this scale. We also describe           executed while building modules. This feature also helps with an
detailed case-studies that illustrate some of the difficulties any au-         effective utilization of resources [47]. Today, CloudBuild is used
tomated technique will need to solve to debug flaky tests arising in           by ≈1200 projects inside Microsoft and executes ≈350 million unit
production. Finally, we develop new tools and techniques to reduce             tests per day across all projects. Therefore, CloudBuild is certainly
the burden of flaky tests on developers, and present interesting               an ideal system to conduct large-scale studies.
challenges that can help guide future research in the area of flaky               Table 1 provides statistics about the flaky tests collected across
tests. Our experience with flaky tests provide three main lessons:             30 days in five projects that use CloudBuild. Due to confidentiality
(1) there is no correlation between the number of flaky tests and              reasons, we anonymized the names of the projects. The goal of
the number of builds that fail due to flaky tests, (2) excessive run-          this data is to show the prevalence of flaky tests in Microsoft. We
time overhead due to factors such as instrumentation can affect                collected this data using a feature of CloudBuild, where failing tests
the reproducibility of flaky tests, and (3) finding differences in the         are automatically reran, and if the rerun passes, we identify the test
                                                                               as flaky. However, there is no guarantee that rerunning a failure




                                                                         102
Root Causing Flaky Tests in a Large-Scale Industrial Setting                                                            ISSTA ’19, July 15–19, 2019, Beijing, China

                         Table 1: Statistics showing the prevalence of flaky tests in five projects using CloudBuild.
                                                     # Test    # Flaky Test       # Distinct     # Builds with Flaky       % of Builds with Flaky
     Projects      # Tests       # Builds       Executions         Failures     Flaky Tests             Test Failures                Test Failures
     ProjA          26,404            302         6,670,299            6,106           2,165                       43                          14%
     ProjB           5,675            430         2,433,452              537             125                      224                          52%
     ProjC          23,651            575         4,596,490            3,530             190                      173                          30%
     ProjD           5,693            741         1,449,233              328              98                      126                          17%
     ProjE           3,390          1,823           786,898            1,564             286                      429                          24%

from a flaky test will result in a pass. Therefore, the actual number                represents a good trade off between gaining logs for both passing
of failures from flaky tests could be higher than the reported values.               and failing executions, and the time spent on test runs. The test
    In the table, Column 2 shows the number of distinct unit tests                   is run offline at a later time instead of on CloudBuild machines,
in each project. Column 3 shows the number of failing builds of                      since running the test 100 times under instrumentation is expensive,
each project. Column 4 shows the number of unit tests executed in                    thereby increasing the build times for the users and causing resource
all builds. Note that CloudBuild does not execute all unit tests in                  contention. Nevertheless, our future work plan includes to not
each build, rather it executes only those tests that are within the                  only collect the passing/failing logs of a flaky test during minimal
modules impacted by the change. Column 5 presents the number of                      workload times on the CI machines, but also to analyze the logs on
unit test failures due to flaky tests, and Column 6 shows the number                 the machines. Doing so would not only notify developers of a flaky
of distinct flaky tests that failed at least once. Finally, Columns 7                test but also the possible root causes of the flakiness as part of their
and 8 show the number and percentage, respectively, of builds that                   normal continuous integration workflow.
contained at least one flaky test failure. Note that each of these                      Since tests may exercise many methods during their execution,
builds can have more than one flaky test failure.                                    these logs can be very large and highlighting the differences among
    Our results show that, although the number of distinct flaky                     the passing and failing executions is beneficial for the majority of
tests is low, the percentage of builds that include flaky test failures              them. Therefore, we also present a tool, RootFinder, that automat-
is substantial. For example, ProjB has the most builds (≈52%) failed                 ically analyzes these logs to highlight the differences to provide
because of one or more flaky test failures, but the project’s number                 developers insight on why the test is flaky.
of flaky tests is substantially lower than other projects (e.g., ProjA).
Upon further investigation into ProjA’s flaky tests, we find that its
                                                                                     3.1    Torch Instrumentation
large number of flaky tests can be attributed to a single change
causing a large number of tests to be flaky for a few builds, and once               Torch [27, 33] is an extensible instrumentation framework for .Net
the flakiness was fixed, these tests no longer caused builds to fail due             binaries. It takes a .Net binary and a set of target APIs, and in-
to flaky test failures. Overall, our results illustrate the prevalence               struments each target API call in the binary. The exact nature of
of flaky tests on these five projects, and how often developers are                  instrumentation depends on what Torch instrumentation plugin is
burdened by flaky tests causing their builds to fail.                                used. For instance, the profiling plugin instruments the binary to
                                                                                     track latencies of APIs executed on critical paths. Torch comes with
                                                                                     plugins for profiling, logging, fault injection, concurrency testing,
3    END-TO-END FRAMEWORK                                                            thread schedule fuzzing, etc. One can extend these plugins or write
We next present our framework to identify the root causes of flaky                   new plugins to suit one’s instrumentation goals.
test failures. The framework consists of multiple steps.                                During instrumentation, Torch replaces each API call with an
    First, CloudBuild identifies flaky tests by rerunning failing tests              automatically generated proxy call, as shown in Figure 1. Note
to see if the retry passes or not. If the retry passes, then the test is             that Torch does not instrument the implementation of a target API;
identified as flaky. CloudBuild then stores information about all of                 only the call to the API is instrumented. The proxy generated by
these flaky tests in a scalable storage.                                             Torch calls the original API; in addition, as shown in Figure 1(c), it
    Next, for each flaky test specified in this storage, we collect all              calls three Torch callbacks—(1) OnStart, called immediately before
dependencies (i.e., test binary, its dependent source binaries, and                  calling the original API, (2) OnEnd, called immediately after the
relevant test data) from CloudBuild, so that the test can be executed                original API returns, and (3) OnException, called when the original
locally on any machine independent of CloudBuild.                                    API throws an exception. OnStart returns a context that is passed
    Next, for each flaky test, we produce instrumented versions of                   to OnEnd and OnException; the context is used to stitch together
all of its dependencies using an instrumentation framework, called                   information tracked by callbacks for the same API call.
Torch [27, 33]. The instrumentation helps us log various runtime                        For identifying the root causes of flaky tests, we use Torch’s
properties of the test execution. Note that instrumented binaries                    logging plugin to passively track and log various runtime properties.
retain the same functionalities as the original binaries, and therefore,             We find that some APIs will behave differently in passing and
tests can seamlessly run on Torch-instrumented binaries.                             failing executions, and analyzing the differences will provide us
    Using the instrumented version, we next run the test 100 times                   insights on the root causes of flakiness. Since such APIs are not
on a local machine in an attempt to produce logs for both passing                    known beforehand, we opt for logging information for all API calls.
and failing executions. The logs generated by Torch contain various                  Specifically, we log the following properties for all API calls.
runtime properties at different execution points. More details are                      (1) Call information, including signature of the API, and its caller
described in Section 3.1. We run the test 100 times as doing so                      API (the API calling the instrumented API). We also track location




                                                                               103
ISSTA ’19, July 15–19, 2019, Beijing, China                                                                   W. Lam, P. Godefroid, S. Nath, A. Santhiar, and S. Thummalapenta


                                                                                            and determines if the method exhibits anomalous runtime behavior
   1    WebClient client = new WebClient();
   2    string data = client.DownloadString (url);
                                                                                            in the failing runs. Examples of input method names that may be of
                                 (a) Original code
                                                                                            interest include methods that return non-deterministic values such
                                                                                            as System.Random.Next (returns a non-negative random integer)
   1    WebClient client = new WebClient();                                                 or System.DateTime.Now (returns a DateTime object representing
   2    TorchInfo ti = Torch.GetInstrumentationInfo();
   3    string data = Torch_DownloadString(ti,client,url);
                                                                                            the current date and time). By default, our framework will use
                                                                                            RootFinder with a predefined set of nondeterministic method calls.
                               (b) Instrumented code
                                                                                            Developers can also add or remove method calls as they wish.
   1    public static string Torch_DownloadString(TorchInfo torchInfo,                          RootFinder works in two steps. In the first step, it processes each
               WebClient instance, string url) {
                                                                                            log file independently and evaluates a set of predicates at each line
   2      string returnValue = null;
   3      var context = Torch.OnStart(torchInfo, instance, url);                            of the log file. The predicates, similar to the ones used in statistical
   4      try {                                                                             debugging [32], determine if the behavior of the callee method in a
   5        returnValue = instance.DownloadString(url);
   6      } catch (Exception exception) {
                                                                                            log line is “interesting” (several example predicates will be given
   7        Torch.OnException(exception, context);                                          shortly). The outputs of the predicates are written to a predicate
   8        throw; // rethrow original exception                                            file. Each line in the predicate file contains the following informa-
   9      } finally {
  10        Torch.OnEnd(returnValue, context);                                              tion about a predicate: (1) The logical epoch when the predicate is
  11      }                                                                                 evaluated, (2) name of the predicate, (3) value of the predicate at
  12      return returnValue;
                                                                                            the current epoch. We currently consider predicates that are local
  13    }
                                 (c) Proxy method                                           to specific code locations, and therefore use logical epochs that
                     Figure 1: Torch instrumentation                                        can identify partial orders of predicates evaluated at the same code
                                                                                            location. More specifically, the epoch is given by a concatenation of
of the API call in the binary and/or source code. Signatures and                            the unique code location of the method call,2 current thread id, and
locations let us uniquely identify each API call.                                           a monotonically increasing sequence number that is incremented
   (2) Timestamp at each call. Timestamps at OnEnd and OnStart                              every time the method is called at the current location. For instance,
give the latency of the API call.                                                           in Figure 3, the method Random.Next() at unique location 9 is called
   (3) Return value at OnEnd and exception at OnException, if any.                          multiple times (e.g., perhaps the line is in a loop or is called by mul-
   (4) A unique Id of the receiver object of the API. This helps                            tiple threads)—once in log line 2 and again in log line 5. Assuming
identifying APIs operating on the same object and thus uncover-                             that they both are executed in the same thread with id 10, the first
ing potential concurrency issues. (5) Ids of the process and thread                         call has the epoch 9:10:1, the second call has the epoch 9:10:2,
executing the API.                                                                          and so on. Partial orders of the epochs can be derived from their
   (6) Id of the parent thread, i.e., the thread that spawned the thread                    ids along with the threads’ parent-child relationship, which Torch
executing the API. The information is important to understand                               dynamically tracks and logs.
dependencies of different threads and their activities [33].
                                                                                            Predicates: A predicate evaluates the state of the method call at
   It is important for the instrumentation to have a small runtime
                                                                                            the current epoch. RootFinder currently implements the following
overhead. Excessive overhead can change runtime behavior by e.g.,
                                                                                            boolean predicates:
removing existing flakiness, introducing new flakiness, or timing
out. The overhead comes from two different sources. First, comput-                               • Relative: The predicate is true if the return value of the cur-
ing some of the runtime properties on the fly can be expensive. For                                rent epoch is the same as that of any previous epochs. This
example, finding signatures of an API and its object type through                                  predicate is useful to identify if a non-deterministic method
reflection, or finding the parent API through stack trace can be                                   is returning the same value in successive calls.
expensive. We avoid this cost by computing these static properties                               • Absolute: The predicate is true if the return value of the
during instrumentation and passing them to the OnStart callback                                    current epoch matches a given value. This predicate is useful
as static parameters (as a TorchInfo object in Line 3 in Figure 1(b)).                             to check if a method returns an error value (e.g., null or an
Second, since we collect runtime information for all APIs, the size                                error code).
of logs they generate can be prohibitively large. To avoid the over-                             • Exception: is true if the method throws an exception.
head, we compress the logs in memory and asynchronously write                                    • Order: The predicate is true if an ordering of method calls
them to disk.1                                                                                     matches a given list of methods and optionally, whether a
                                                                                                   specified amount of time occurred between the methods.
3.2     Log Analysis Tool                                                                          This predicate is useful to identify thread interleavings.
                                                                                                 • Slow: This predicate is true if the method call takes more than
We develop a simple tool called RootFinder to parse Torch logs of
                                                                                                   a specified time. (The threshold can be determined based on
passing and failing executions to identify potential root causes of
                                                                                                   domain knowledge of the called method, or by analyzing
certain types of flaky tests. At a high level, RootFinder takes as input
                                                                                                   latencies of passing test runs.)
a method name that is likely to be the cause of the flakiness and two
directories containing Torch logs of passing and failing test runs,
                                                                                            2 Unique code location uniquely identifies the location of a method call in the code. An
1 We also experimented with more lightweight Event Tracing for Windows (ETW)                example is the name of the program source/binary file plus the line number/binary
logging [1]; however, at a high logging event rate, ETW may skip logging randomly           offset of the method call within the file. For simplicity we use Source# as the unique
chosen events. We observed a high loss rate, and hence did not use ETW for logging.         location in the rest of the paper.




                                                                                      104
Root Causing Flaky Tests in a Large-Scale Industrial Setting                                                               ISSTA ’19, July 15–19, 2019, Beijing, China


     • Fast: This predicate is true if the method call takes less than             1     class TestAlertTest {
                                                                                   2       void TestUnhandledItemsWithFilters() {
       a specified time.                                                           3         TestAlert ta1 = CreateTestAlert();
                                                                                   4         TestAlert ta2 = CreateTestAlert();
   After the predicate files are generated, RootFinder compares                    5         ...
all predicate files (from passing and failing runs) to identify ones               6         Assert.AreNotEquals(ta1.TestID, ta2.TestID);
                                                                                   7       }
that are true/false in all passing executions, but are the contrary                8       TestAlert CreateTestAlert() {
in all failing executions. Intuitively, these predicates are strongly              9         int id = new Random().Next();
correlated to test failures and hence are useful to understand the un-            10         ...
                                                                                  11         return new TestAlert(TestID = id, ...);
derlying root cause of failures. Specifically, RootFinder labels each             12       }
predicate in the predicate files with one of the following categories:            13     }

(1) Inconsistent-in-passing: Such a predicate either appears in                        Figure 2: Test method from a Microsoft product’s test suite.
only a subset of all passing test runs or appears in all passing runs
but with more than one value. The log line corresponding to such a
predicate is likely irrelevant as to why a test is flaky. This is because
whether the predicate was true or false did not affect the outcome
of the test runs (i.e., they always passed).
(2) Inconsistent-in-failing: Such a predicate either appears in
only a subset of all failing test runs or appears in all failing runs but
with more than one value. As in the case of the previous category,
this predicate is also likely irrelevant as to why a test is flaky.               Figure 3: Torch logs for passing test executions of the test in
(3) Consistent-and-matching: Such a predicate appears in all                      Figure 2. TUIWF is TestUnhandledItemsWithFilters.
passing and failing runs and with the same value. This predicate is
also likely irrelevant as to why the test is flaky as it did not affect
the final outcome of the test runs.
(4) Consistent-but-different: Such a predicate either (I) appears
only in passing or only in failing runs, or (II) is true in all passing
runs but false in all failing runs (or vice versa). (I) indicates that
executions of a passing and a failing run diverge before the epoch
where the predicate was evaluated (which is why the predicate
appears in one set and not the other), while (II) indicates how a
method consistently behaves differently in the passing and failing                Figure 4: Torch logs for failing test executions of the test in
runs. This predicate is highly likely to explain why a test is flaky              Figure 2. TUIWF is TestUnhandledItemsWithFilters.
because it precisely shows how passing and failing test runs differ.
   By default, the predicates outputted by RootFinder are sorted so               An Example. Figure 2 shows the simplified version of a test of a
that the ones that are most likely to explain why a test is flaky are             confidential product in Microsoft (the corresponding code under
shown first (i.e., Consistent-but-different predicates). Once the cate-           test implementing TestAlert is omitted for brevity).
gories are sorted, RootFinder then sorts the predicates within each               TestUnhandledItemsWithFilters is flaky since new Random().Next()
category so that the predicates with the lowest log line numbers                  may actually return the same value if two consecutive calls are
are outputted before the ones with higher numbers. In our case                    invoked close together. This is because if a Random object is instanti-
studies with RootFinder as described in Section 4.2, we find that                 ated without a seed, it takes the current system time as the default
sorting predicates as stated enables RootFinder to output useful                  seed; therefore, two Random objects instantiated without a seed and
ones in the least amount of time.                                                 within a short window of time may be initialized with the same
   The predicates outputted by RootFinder can aid the debugging                   seed, causing their Next() calls to return the same sequences of
efforts of nine out of ten categories of flaky tests mentioned in a               random numbers.
survey [34]. More specifically, for categories such as Network, Time,                Figures 3 and 4 show a fragment of Torch logs from passing
IO, Randomness, Floating Point Operations, Test Order Dependency,                 test runs and from failing test runs (respectively) of the test in
and Unordered Collections, RootFinder can directly compare the                    Figure 2. As shown in Lines 2 and 5 of Figure 4, the StartTimes of
return value of failing and passing Torch logs to identify predicates             the Random.Next() calls are the same in all failing logs, therefore
that are highly likely to explain why the test is flaky. For the Async            the return value for both calls to Random.Next() is the same value
Wait and Concurrency categories, our framework currently relies                   (e.g., 21, 17, and 5). In the passing logs such as the ones depicted
on Torch’s ability to first fuzz delays, and then for RootFinder to               in Figure 3, we can see that on Lines 2 and 5, StartTimes are differ-
identify latency-related predicates, such as Fast and Slow, to help               ent and consequently, the return values of Random.Next() are also
developers root cause those categories. For the remaining category,               different. The assertion on Line 8 of Figure 2 passes if the return
Resource Leak, we plan to extend the set of predicates to include                 values of Random.Next() are different and fails otherwise.
memory leak detection tools [5, 9] to help developers understand                     RootFinder can narrow down the above root cause when it is
tests of this category with new predicates.                                       invoked for the method Random.Next(). In step 1 of its processing, it




                                                                            105
ISSTA ’19, July 15–19, 2019, Beijing, China                                                     W. Lam, P. Godefroid, S. Nath, A. Santhiar, and S. Thummalapenta

          Table 2: Characteristics of the specific flaky test examples in Section 4.2 and of all 44 flaky tests in our dataset.
                                    Duration            % of failed   # of method           # of unique        # of threads       # of objects
                                       / test       executions/test      calls/test     method calls/test             / test             / test
              Specific examples in Section 4.2
              Time                   1s                        29%             4.7k                    463                   3             1,151
              Randomness             4s                        91%             0.8k                    182                   1               249
              Async Wait             2s                         1%             0.2k                     90                   4                62
              Concurrency           10s                         1%             18k                     882                   8             8,200
              Resource Leak          4s                         1%             0.6k                    218                   6               246
              All 44 flaky tests
              Median                           5s               6%             2.5k                    248                   3               637
              Average                         45s              28%            335k                     335                   5            55,418

converts the passing logs in Figure 3 into predicate files containing             in the flaky failures. These failures can be broadly classified into
the predicate (9:2, Relative, False). This predicate means that the               three categories; (1) the Torch instrumentation resulted in a new
method Random.Next() in Source# 9 and Seq# 2 returns a value that is              failure, (2) there are some issue in the instrumentation part of our
different from the return value of the immediate previous call of the             framework that needs to be fixed, or (3) there are differences in
same method at the same Source#. Similarly, it converts the failing               CloudBuild and the test machine where we ran the tests. Our future
logs in Figure 4 into predicate files containing the predicate (9:2,              work plan is to investigate these issues and also try alternatives
Relative, True). In step 2, RootFinder compares the predicates                    such as running the instrumented tests directly on CloudBuild.
across runs and identifies the predicate as Consistent-but-different.
This predicate quickly points to a root cause (or a symptom that is               4.1     Study Dataset
strongly correlated to the root cause) of the flakiness, as well as its
                                                                                  We use a dataset consisting of 44 flaky tests. They belong to 22
code location (encoded in the epoch of the predicate).
                                                                                  software projects from 18 Microsoft internal/external products and
                                                                                  services. For each test, the dataset contains 100 execution traces,
4    CASE STUDIES                                                                 some of which are from failed executions. Each trace file consists
                                                                                  of a sequence of records containing various runtime information
We next present the results of applying our framework on large
                                                                                  about an executed method as described in Section 3.1.
projects that use CloudBuild as their CI pipeline. To ensure that
                                                                                     Table 2 shows some characteristics about the tests and traces.
our results are not biased due to a single project, we collected all
                                                                                  The characteristics show the overall complexities of the tests. The
distinct flaky tests recorded during a day in our production en-
                                                                                  average run duration of the tests is nontrivial (45s), even though
vironment. Among these flaky tests, we identified the tests that
                                                                                  they are all unit tests and most of the I/O calls are mocked with fast
are compatible with the Torch instrumentation framework. More
                                                                                  proxy calls. Each test runs a large number of methods (335k total
specifically, CloudBuild supports unit tests written in both managed
                                                                                  methods/test and 335 unique methods/test), mostly because a tested
(such as C#) and unmanaged (such as C++) code [4]. Also, Cloud-
                                                                                  component often depends on many underlying components, each
Build supports tests written for various test frameworks such as
                                                                                  invoking many methods. 80% of the tests use more than one thread
MsTest [6], NUnit [7], and XUnit [10]. Our current implementation
                                                                                  and on average, each test runs on 5 threads and operates on 55418
of the instrumentation framework supports only unsigned (binaries
                                                                                  objects. Many of these elements can introduce nondeterminism
that do not include digital signatures) and managed code, and is
                                                                                  that can make a test flaky. Moreover, the tests produce massive
also tailored for those tests that run using the MsTest framework.
                                                                                  runtime logs, which can be extremely challenging to analyze.
    Overall, we collected 315 flaky tests that matched the criteria de-
                                                                                     Each runtime log in our dataset contains a wealth of information.
scribed above. Our collected flaky tests belong to different projects
                                                                                  For example, it contains all methods executed by the test, their
that provide Microsoft services for both internal and external cus-
                                                                                  latencies, return values, parent-child relationships of threads, ac-
tomers, and also fall into different categories such as database,
                                                                                  tivities of threads, etc. We believe that the dataset will be useful to
networking, security, and core services. Among these flaky tests,
                                                                                  the research community, not only to conduct research on various
we were only able to reproduce flakiness i.e., produce logs for both
                                                                                  aspects of flaky tests and their root causes, but also for a general un-
passing and failing executions in 100 runs for 44 tests. Among the
                                                                                  derstanding of runtime behavior of tests in a production system. We
remaining tests, 97 of them have all of their runs pass. For these
                                                                                  have, therefore, made an anonymized version of the dataset avail-
tests, we also tried a fuzzing technique that introduces delays using
                                                                                  able to the public [8]. In the anonymized dataset, sensitive strings
Torch, however we were still unable to reproduce the flakiness. It
                                                                                  (such as method names containing Microsoft product names) are
is important to note that the focus of our framework is for identify-
                                                                                  replaced with hash values. The hashes are deterministic and hence
ing and debugging flaky tests. Our findings here that only 44 out
                                                                                  can be correlated within and across trace files.
of 315 flaky tests are reproducible suggests that improvements to
reproducing flakiness can also be highly impactful.
    For the remaining 174 tests, we find that all 100 runs failed.                4.2     Case Studies of Finding Root Causes
During our inspection, we find that for some of the cases, the tests              As explained in Section 3.2, our framework in theory can address
failed with a different error signature than the one that resulted                nine out of ten categories of flaky tests. In this section, we provide




                                                                            106
Root Causing Flaky Tests in a Large-Scale Industrial Setting                                                                ISSTA ’19, July 15–19, 2019, Beijing, China


 1    [TestMethod]                                                                    1   [TestMethod]
 2    public void TestReplicaService() {                                              2   public void DelayedTaskStaticBasicTest() {
 3      ...                                                                           3     int delay = 1000; int i = 0;
 4      byte[] response = Service.SendAndGetResponse(payload);                        4     Scheduler.ScheduleTask(
 5      byte[] replicaResponse = ServiceReplica.SendAndGetResponse(payload);          5       DateTime.UtcNow.AddMilliseconds(delay),
 6      Assert.AreEqual(response, replicaResponse);                                   6       new LoggedTask(
 7    }                                                                               7         "TestDelayedTaskFrameworkTask", () => { i = 1; },
 8    public class Service : NetworkService {                                         8         new Dictionary<string, string> { { "test", "value" } }));
 9      public byte[] SendAndGetResponse(Request req) {                               9     Thread.Sleep(500);
10        ...                                                                        10     Assert.IsTrue(i == 0);
11        DateTime currentTime = DateTime.UtcNow;                                    11     Thread.Sleep(delay);
12        Message message = new Message(req, currentTime);                           12     Assert.IsTrue(i == 1);
13        return base.SendAndGet(message.Serialize());                               13   }
14      }
15      ...                                                                          Figure 6: A test that is flaky due to waiting for asynchronous calls.
16    }

     Figure 5: A test that is flaky due to getting the system time.                  Random.Next() actually uses the system time as the seed to gen-
                                                                                     erate a random number if no seed is provided by the user. There-
                                                                                     fore, when the timing delay between the CreateTestAlert() calls
                                                                                     and consequently the calls to new Random().Next() is too small for
in-depth examples of flaky tests and how RootFinder assisted devel-                  the system to record that it’s different, then the test will fail. In
opers with debugging these particular flaky tests. The remainder                     our experiments, we find that the TestUnhandledItemsWithFilters
of this section presents four examples of flaky tests that our frame-                test fails 91 out of 100 times. The fix for the flakiness of this test
work can find root causes for and one example that our framework                     should be for the CreateTestAlert helper test method to not use
cannot. All examples are anonymized and simplified as needed.                        random numbers for testID. Instead CreateTestAlert should take
                                                                                     an int parameter and allow tests to pass in values for testID so
4.2.1 Time. We find some tests to be flaky due to improper use of                    that TestAlerts that should and shouldn’t have the same TestID
APIs dealing with time. These flaky tests rely on the system time,                   can be decided by the method calling CreateTestAlert. A useful
which introduces non-deterministic failures, e.g., a test may fail                   predicate for this example is described in Section 3.2. When we
when time zones change.                                                              apply RootFinder to this example without any domain knowledge
   Figure 5 shows a simplified version of a test case, which ensures                 from developers, it took, on average, 2 seconds to run and outputted
that a service and its replica return the same response to a particu-                408 predicates total. The useful predicate was ranked first.
lar message. A developer may observe that the assertion on Line 6
occasionally fails. The failures are because calls to Service and                    4.2.3 Async Wait. Tests are flaky due to the Async Wait issues
ServiceReplica’s SendAndGetResponse may or may not use the same                      when the test execution makes an asynchronous call and does not
timestamps. If the invocations of SendAndGetResponse on Lines 4                      properly wait for the result of the call to become available before
and 5 happen within a short window of time, the timestamps pro-                      using it. Depending on whether the asynchronous call was able to
duced by DateTime.UtcNow on Line 11 can be the same due to the                       finish execution or not, such flaky test may pass or fail.
limited granularity of the system timer. The granularity is seconds                     Figure 6 shows an example of an Async Wait flaky test.
by default. If the timestamps are the same, then the test passes;                    DelayedTaskStaticBasicTest schedules a task to run at a pre-defined
otherwise, it fails. We find many flaky tests at Microsoft exhibiting                time (the current time + 1000 milliseconds) on Line 4. In our exper-
similar behavior. This example fails 29% of the time in our experi-                  iments, we find that the DelayedTaskStaticBasicTest test fails 99
ments, but our experiments also find other tests exhibiting a similar                out of 100 times. The test can be flaky due to two main reasons.
root cause to fail up to 88% of the time.                                               (1) When the asynchronous task on Line 4 finishes executing
   A useful predicate for this example should indicate that the                      before Line 10, then the test will fail. In passing executions of this
timestamp of SendAndGetResponse when invoked by Line 5 is always                     test the value of i has not changed to 1, but in the failing executions,
the same as the timestamp when invoked by Line 4 in the passing                      delays before the assertion on Line 10 can actually be greater than
logs, but they are always different in the failing logs. When we apply               the time it takes to execute the asynchronous task on Line 4. In
RootFinder to this example without any domain knowledge from                         such cases, the value of i when Line 10 executes is already 1 and
developers, it took, on average, 11 seconds to run and outputted                     the assertion will fail.
1163 predicates total. The useful predicate was ranked at 81. In                        (2) When the assertion on Line 12 finishes executing before the
practice, when developers used RootFinder on this example, they                      asynchronous task on Line 4 finishes executing, then the test will
were able to input suspicious method names to quickly find the                       fail. In passing executions of this test, the value of i is changed
useful predicate in a matter of minutes.                                             to 1 before Line 12 executes, but in the failing executions, the
                                                                                     asynchronous task on Line 4 runs so slow that the delays on Lines 9
4.2.2 Randomness. Tests may pass or fail if their results depend on                  and 11 are not enough to prevent Line 12 from executing before the
random numbers. More specifically, tests that use a random number                    asynchronous task finishes. In all 6 test failures that we encounter,
generator without accounting for all the possible values that it may                 the flaky test failed due to this second reason.
generate can be flaky.                                                                  A useful predicate for this example should indicate that the task
   One example of such a flaky test we find is shown in Figure 2.                    on Line 4 always took longer in the failing runs. When we apply
As explained in Section 3.2, the test in Figure 2 is flaky since                     RootFinder to this example without any domain knowledge from




                                                                               107
ISSTA ’19, July 15–19, 2019, Beijing, China                                                      W. Lam, P. Godefroid, S. Nath, A. Santhiar, and S. Thummalapenta


developers, it took, on average, one second to run and outputted                  1    [TestMethod]
                                                                                  2    public async Task TestDirtyResource() {
868 predicates total. The useful predicate was ranked at 17.                      3      ...
                                                                                  4      using (var emptyPoolManager = CreatePoolManager(...)) {
4.2.4 Concurrency. A flaky test’s root cause is Concurrency when                  5        Resource pm =    ResourceUtils.CreatePhysicalMachine(...);
the test can pass or fail due to different threads interacting in a               6        await emptyPoolManager.AddOrUpdateResourceAsync(pm, HeartbeatStatus.
non-deterministic manner (e.g., data races, deadlocks).                                           InUse);
                                                                                  7        ...
   Figure 7 shows an example of a test that is flaky due to concur-               8        for (int i = 0; i < 5; i++) {
rency issues. The TestDirtyResource method tests that a manager                   9          var sessionId = Guid.NewGuid().ToString();
                                                                                 10          var request = new ResourceAllocateRequest(pm);
(created in Line 4) of a cluster properly recycles used resources.               11          await emptyPoolManager.PreAllocateResourcesAsync(request.Yield(), pm.
Once the manager is created, it is setup by adding a machine to it,                                 Specification, TenantId, sessionId, sessionPriority);
that is in use, or “dirty” (Lines 5–6), along with more setup code               12          var response = (await QueryAllocate(emptyPoolManager, request.Yield()
                                                                                                    , TenantId, sessionId)).FirstOrDefault();
(omitted for brevity). The test then repeatedly creates and sends                13          Assert.IsNotNull(response);
requests to the cluster to execute jobs (Lines 9–12), and makes sure             14          Assert.AreEqual(request.Id, response.RequestId);
that the response obtained in line 12 is correct (Lines 13–16). Finally,         15          Assert.IsNotNull(response.ResourceId);
                                                                                 16          Assert.AreEqual(pm.ResourceId,response.ResourceId);
the resources used by the cluster to process the particular request              17          await emptyPoolManager.ReleaseResourcesAsync(response.ResourceId);
are released (Line 17), and the newly freed resource heartbeats                  18          await emptyPoolManager.HeartbeatResourceAsync(pm, HeartbeatStatus.
                                                                                                    Ready);
its status to the manager (Line 18), and the manager processes it                19          await emptyPoolManager.ProcessResourcesHeartbeats(CancellationToken.
(Line 19). We observed in our experiments that the assertion on                                     None);
Line 13 failed occasionally.                                                     20        }
                                                                                 21      }
   Upon investigating the failure, we find that the creation of the              22    }
resource manager (Line 4) also starts a background task that pe-
                                                                                           Figure 7: A test that is flaky due to concurrency.
riodically marks resources as unavailable in case T milliseconds
(ms) has elapsed since the last heartbeat was processed. Test failure             1    DatabaseProvider ssp;
occurs when this task runs T ms after another thread has processed                2    String currentDirectory = ...;
Line 19, since the background task would mark the cluster resource                3    [TestMethod]
                                                                                  4    public void ResourceAllocation() {
as unavailable, which would then cause the subsequent resource                    5      ssp = new DatabaseProvider(currentDirectory);
allocation request made using QueryAllocate on Line 12 to return                  6      ...
null. The null value will then cause the assertion on Line 13 to fail.            7    }
                                                                                  8    [TestCleanup]
This example demonstrates a subtle flakiness condition that only                  9    public void TestCleanup() {
manifests on a particular thread interleaving, and moreover, only if             10      ClearConnections();
                                                                                 11      if (File.Exists(dbPath)) {
the interleaving follows very precise timing constraints.                        12        File.Delete(dbPath);
   A useful predicate for this example should indicate that the                  13      }
background task from Line 4 always ran after Line 19 and it always               14      ...
                                                                                 15    }
runs T ms or longer after Line 19. When we apply RootFinder to this
example without any domain knowledge from developers, it took,                            Figure 8: A test that is flaky due to resource leaks.
on average, 126 seconds to run and outputted 127187 predicates
total. The useful predicate was ranked at 3231. In practice, when                collector does not run first, then the file resource is held, causing
developers used RootFinder on this example, they provided some                   the subsequent attempts to delete the file on Line 12 to throw an
domain knowledge and RootFinder was eventually able to help                      exception. Since our framework relies on Torch to capture infor-
them debug the root cause of this flaky test. The difficultly for                mation relevant to the root cause of the flaky test from the user’s
RootFinder here is largely because this flaky test’s root cause is not           code, our framework is currently unable to assist developers for
only thread interleaving, but also the timing of the interleavings.              Resource Leak flaky tests such as the one in this example.
Our future work plan includes improving RootFinder to consider
combinations of predicates (e.g., Order and Slow) to better root                 4.3     Applying RootFinder on to Case Studies
cause flaky tests such as this example.
                                                                                 When we apply RootFinder onto the examples described in Sec-
4.2.5 Resource Leak. A flaky test’s root cause is Resource Leak                  tion 4.2, we find that the root causes are concisely summarized as
when the test passes or fails because the application does not prop-             predicates in the output for four of the five examples, especially
erly acquire or release its resources, e.g., locks on files.                     when developers provided domain knowledge to RootFinder. These
   Figure 8 shows an example of a Resource Leak flaky test.                      predicates can assist developers in identifying what values are the
ResourceAllocation tests whether the necessary resources are prop-               same or not in the passing and failing executions, and provide them
erly allocated for an application. The application internally uses a             the code location that produced these values. When we present the
third-party database to store some information. The TestCleanup                  findings of RootFinder to developers or use it ourselves, they/we
method is executed after every test case to delete the database, so              are able to more quickly reproduce the failures of the flaky test.
that it may be re-initialized before the next test case. Although                   For the remaining example, Resource Leak, our future work
Line 12 tries to close the connection to the database, the third-party           plan includes possibly extending the end-to-end framework, Torch
library requires the garbage collector to run prior to this step in              instrumentation framework, and RootFinder to provide valuable,
order to release the file handle to the database file. If the garbage            concise information to developers in a timely manner. For example,




                                                                           108
Root Causing Flaky Tests in a Large-Scale Industrial Setting                                                      ISSTA ’19, July 15–19, 2019, Beijing, China


one possible extension we plan to make is to extend the end-to-                 or fail with the same version of code. Due to the nondeterministic
end framework to include memory leak detection tools [5, 9]. This               nature of these tests, the flaky tests from our dataset that we are
extension will allow us to provide developers information for the               able and unable to produce Torch logs for may not be true if the
Resource Leak example in Section 4.2.5. Integrating such an exten-              experiment was to be repeated. To mitigate this threat, we ran each
sion may impose significant performance, resource and time cost                 test a nontrivial amount of times.
to our current framework though. Therefore, more research and
experimentation may be required to determine the best solutions
to address all of the possible root causes of flaky tests.                      7   RELATED WORK
                                                                                Prevalence and costs of flaky tests. Luo et al. [34] noted in first
5    LESSONS LEARNED                                                            extensive study of flaky tests that specific numbers on the occur-
                                                                                rence of flaky tests were hard to obtain. Since then, the problem
This section summarizes the main lessons learned so far from our
                                                                                has received more attention, with recent results indicating that
experience with flaky tests in our specific industrial setting.
                                                                                flaky test failures are relatively common. We find that on average,
   (1) Reproducing flakiness locally is challenging. In our experi-
                                                                                27.4% builds exhibit flaky tests while Labuschagne et al. [30] found
ments, we tried 315 tests that showed flaky behavior in CloudBuild.
                                                                                that 12.8% of builds in their data set of 935 builds from 61 projects
However, when run on a machine outside CloudBuild, we did not
                                                                                using Travis CI, a cloud-based CI tool, failed because of flaky tests.
observe flakiness in a vast majority of them (86%), even after run-
                                                                                Palomba and Zaidman [42] found 8829 flaky tests out of 19532 JUnit
ning each test 100 times.
                                                                                tests from 18 projects - 45% of all tests they analyzed were flaky.
   (2) Runtime overhead from instrumentation can affect the repro-
                                                                                Moreover, these tests showed both passing and failing behavior
ducibility of flaky tests. We used several runtime optimizations in
                                                                                using only up to 10 reruns. In our experience, flaky behavior is
Torch, including pre-computing static properties and compressing
                                                                                rarer (around 4.6% of all tests flaked), as well as harder to repro-
logs. Yet, when we randomly sampled 59 flaky tests to run 100 times
                                                                                duce. Of 311 tests in our dataset that failed on the cloud, we could
locally on a machine with and without Torch instrumentation, we
                                                                                not reproduce the failure locally in 97 cases, even with 100 reruns.
observed that 2 tests are flaky only with Torch instrumentation,
                                                                                Rahman and Rigby [43] showed, using Firefox as a case study, that
while 3 tests are flaky only without Torch instrumentation.
                                                                                ignored flaky tests can lead to crashes in production. In spite of
   (3) The number of flaky tests a project contains does not correlate
                                                                                this, developers do ignore flaky tests; Thorve et al. [49] examine 77
to the frequency in which builds fail. E.g., as shown in Table 1, ProjB
                                                                                commits pertaining to flaky tests from 29 Android projects, finding
has a low number of flaky tests but a high number of failed builds,
                                                                                that 13% of the commits simply skipped or removed flaky tests. Lam
whereas the situation is the opposite for ProjA.
                                                                                et al. [31] found 388 flaky tests in 683 projects, and publicized the
   (4) Finding differences in the runtime behavior of flaky tests can
                                                                                list flaky tests they found online. Their dataset includes the names
be an effective way to help developers root cause these tests. In fact,
                                                                                and partial classification of the flaky tests, but does not contain
our preliminary tool along with our lightweight instrumentation
                                                                                detailed execution logs, such as the ones provided in our dataset.
can already help developers in nine out of ten causes of flaky tests.
                                                                                Causes of flaky tests. Zhang et al. [51] note that test suites can suf-
   We emphasize that these results are preliminary and that the
                                                                                fer from test order dependency where test outcomes can be affected
main purpose of this paper is instead to encourage more work
                                                                                by the order in which the tests are run. In our case, CloudBuild
in this specific problem area. With this goal in mind, we identify
                                                                                always runs tests of a test suite in the same order. Luo et al. [34]
several open research challenges in Section 8.
                                                                                investigate 201 commits from 51 open-source projects, finding that
                                                                                the primary causes for flakiness are (i) async-wait, (ii) concurrency,
6    THREATS TO VALIDITY                                                        including atomicity violation, data races and deadlocks, and (iii) test
Internal Validity: Our threats to internal validity are concerned               order dependency. Thorve et al. [49] find additional root causes for
with our study procedure’s validity. RootFinder and our framework               flaky tests in Android. These studies worked backwards from the
can contain faults that impact our results and lessons learned. We              commits fixing flaky tests to ascertain the root cause; in contrast,
attempt to mitigate this threat by having thorough code reviews                 our RootFinder aims to diagnose flakiness before the flaky tests are
and testing of RootFinder and our framework. Furthermore, we rely               fixed. Gao et al. [20] observe that it is difficult to reliably replay
on various other tools in our framework, such as Torch. These tools             and reproduce outcomes from tests where a user interacts with
could have faults as well and such faults could have also affected              a system, owing to test dependencies on system platform, library
our results. To mitigate this threat, the logs produced by Torch and            versions and tool configurations. Although we focus on unit tests,
the root causes produced by RootFinder are manually analyzed and                our experience has been similar.
confirmed by at least two of the authors.                                       Detecting and fixing flaky tests. A standard strategy in CI pipelines
External Validity Our threats to external validity are concerned                to deal with flaky tests is to rerun them—if a failing test passes on
with all threats unrelated to our study procedure. Our lessons                  re-running, it no longer gates a build [34, 37]. However, running
learned may not apply to projects other than the ones in our study.             tests can be expensive especially when these tests are gating a build.
We attempt to mitigate this threat by including a diverse range of              DeFlaker [13] monitors the code coverage of recent changes and
projects in our study. Our projects service for both internal and               marks as flaky any newly failing test that did not execute any of the
external customers of Microsoft, and also fall into different cate-             changes. On the other hand, RootFinder helps developers root cause
gories such as database, networking, security, and core services.               flaky tests by finding the differences in the logs of passing and fail-
Flaky tests are, by definition, tests that nondeterministically pass            ing executions. Since Deflaker helps developers know which tests




                                                                          109
ISSTA ’19, July 15–19, 2019, Beijing, China                                                    W. Lam, P. Godefroid, S. Nath, A. Santhiar, and S. Thummalapenta


may be flaky and RootFinder helps developers root cause flaky tests,             software components or only in some? If some, which ones and
we recommend developers to first use Deflaker to find flaky tests                why? Should function input and output values be logged as well?
and then use RootFinder to root cause them once they are found.                  Should intra-procedural execution information (e.g., which code
Herzig and Nagappan [25] propose an association rule mining based                instructions, blocks or branches were executed) also be recorded
technique to tag a system/integration test with many steps that                  for a fine-grained analysis?
fails as a false alarm, based on whether each step failed or passed.                Logging versus analysis tradeoffs. The more data is being
Among approaches to detect root causes of flaky tests, detecting de-             logged, the more computationally expensive it is to store and pro-
pendent tests has received the most research attention [31, 38, 51].             cess all the data. The analysis itself becomes more complicated:
Palomba and Zaidman [42] established a relationship between test                 non-essential differences may creep in if too much information is
smells and flaky tests. Refactoring the tests fixes the flakiness in             recorded, which makes it harder to identify meaningful differences.
all the tests containing the code smell in their study. However, it is           On the other hand, recording too little information may omit key
not clear how to generalize their approach to other root-causes for              events explaining the source of the test flakiness. How to strike the
flaky tests. Shi et al. [46] detected nondeterministic flaky tests by            right balance between logging and analysis effectiveness is another
detecting tests that assumes a deterministic implementation with a               key challenge in this space.
nondeterministic specification. In comparison to our work, we do                    Fixed versus variable logging. Another dimension to the prob-
not require specifications to detect flaky tests.                                lem is whether the level of detail used for logging should be the
    Concurrency is a major root-cause for flaky tests [34, 42, 49],              same for all applications and tests, or whether it should be adjusted,
therefore detecting and fixing concurrency issues could partially ad-            automatically over time in an “iterative-refinement process”, or
dress the problem. There exist automated techniques to detect data-              using user-feedback. If the logging level can be adjusted, should it
races [18, 39], deadlocks [21, 40, 44] and atomicity violations [17].            start lazily, at a high-level and be refined until meaningful differ-
However, to integrate these techniques within modern CI pipelines                ences are detected? Or, should logging proceed bottom-up, eagerly
would require overcoming significant challenges of meeting per-                  logging many events, then identifying irrelevant details (data noise)
formance, resource and time constraints, providing multi-language                and eliminating those until meaningful differences are found?
support, and controlling rates of false positives and false nega-                   Logging versus reproducibility. The more data one logs, the
tives [14, 15, 28, 35]. Given the ubiquity of flaky tests in modern              more intrusive the runtime instrumentation can be. As discussed
continuous integration environments, Harman and O’Hearn [24]                     earlier in this paper, reproducing flakiness is hard, and can become
advocate assuming all tests as flaky (with some probability).                    even harder when significant runtime slowdowns are introduced
    Like RootFinder, several other systems use differences in runtime            by expensive logging activities.
invariants in passing and failing tests to identify likely causes of
failures [16, 22, 23, 32, 48]. Fault localization techniques and tools,          9   CONCLUSION
such as Tarantula [29], Ochiai [12], Op2 [41], Barinel [11], and
                                                                                 Flaky tests are a prevalent issue plaguing large software develop-
DStar [50], analyze different passing and failing tests in order to
                                                                                 ment projects. Simply ignoring flaky tests from a regression suite
localize likely faults in programs. In contrast, RootFinder focuses
                                                                                 is a dangerous practice since it might mask severe software defects
only on non-deterministic tests that sometimes pass and sometimes
                                                                                 that will hurt users later on. To help software developers and testers,
fail, it depends on collecting a large volume of runtime logs (to not
                                                                                 we clearly need better tools and processes for dealing with flaky
miss rare flakiness and to check for unplanned invariants), and it
                                                                                 tests. This paper presented two main efforts currently under way at
optimizes the process of log collection (to not affect flakiness and
                                                                                 Microsoft to address flaky tests. The first part consists of a motiva-
reproduce it even under instrumentation).
                                                                                 tional study and a dataset of flaky tests. The second part consists of
                                                                                 a framework and preliminary tool to help debug flaky tests. More
8    OPEN RESEARCH CHALLENGES                                                    specifically, we described how centralized software building and
                                                                                 testing can help detect and manage flaky tests. We also discussed
The preliminary results reported in this paper are just scratching
                                                                                 preliminary work on new tools for root-causing flaky tests, in order
the surface of identifying the root causes of flaky tests. By sharing
                                                                                 to determine in a cost-effective manner what causes a test to be
our dataset of flaky tests, we hope to encourage more research in
                                                                                 flaky and how to eliminate that root cause. The purpose of this
this area. Here are open questions that are left to be explored.
                                                                                 paper is also to share with the research community a large data
   How to evaluate results. A major challenge is to determine
                                                                                 set of flaky tests and the tools we are currently building to root-
the “ground truth” of why a test is flaky. Ultimately, this requires de-
                                                                                 cause these tests. We hope that this paper, dataset and tools will
velopers to look at each flaky test, examine the suggested candidate
                                                                                 encourage more research on this important problem.
root causes, and then decide which root cause(s) is/are the most
likely. For a large number of flaky tests, this evaluation is expensive.
Moreover, results may vary depending on the developer’s expertise                ACKNOWLEDGMENTS
and familiarity with the code being tested and with the tests being              Most of the work of Wing Lam was done while visiting Microsoft.
performed. How to prevent biased evaluations due to diversity in                 We thank Ankush Das for his contributions to an early version of
developer expertise is another non-trivial challenge.                            this work. We also thank August Shi, Owolabi Legunsen, Tao Xie,
   What information to log. Earlier in the paper, we showed                      and Darko Marinov for their discussions about flaky tests. We also
one way to log execution traces. But there are many other op-                    acknowledge support for research on flaky tests and test quality
tions. Should all method calls and returns be logged? In all the                 from Huawei and Microsoft.




                                                                           110
Root Causing Flaky Tests in a Large-Scale Industrial Setting                                                                                  ISSTA ’19, July 15–19, 2019, Beijing, China


REFERENCES                                                                                         [29] J. Jones and M. Harrold. 2005. Empirical evaluation of the Tarantula automatic
 [1] 2011. Event Tracing for Windows (ETW) simplified. https://bit.ly/2NgEBzl.                          fault-localization technique. In ASE. Long Beach, CA, USA, 273–282.
 [2] 2019. Bazel. https://bazel.build.                                                             [30] A. Labuschagne, L. Inozemtseva, and R. Holmes. 2017. Measuring the cost
 [3] 2019. Buck. https://buckbuild.com.                                                                 of regression testing in practice: A study of Java projects using continuous
 [4] 2019. Managed vs. Unmanaged development. https://bit.ly/2Or4C3p.                                   integration. In ESEC/FSE. Paderborn, Germany, 821–830.
 [5] 2019. MemorySanitizer. https://clang.llvm.org/docs/MemorySanitizer.html.                      [31] W. Lam, R. Oei, A. Shi, D. Marinov, and T. Xie. 2019. iDFlakies: A framework for
 [6] 2019. MsTest framework. https://bit.ly/2NeZmLO.                                                    detecting and partially classifying flaky tests. In ICST. Xi’an, China, 312–322.
 [7] 2019. NUnit framework. https://nunit.org.                                                     [32] B. Liblit, M. Naik, A. Zheng, A. Aiken, and M. Jordan. 2005. Scalable statistical
 [8] 2019. RootFinder. https://sites.google.com/view/root-causing-flaky-tests/home.                     bug isolation. ACM SIGPLAN Notices 40 (2005).
 [9] 2019. Valgrind. http://valgrind.org.                                                          [33] L. Luo, S. Nath, L. Sivalingam, M. Musuvathi, and L. Ceze. 2018. Troubleshooting
[10] 2019. xUnit framework. https://xunit.github.io.                                                    transiently-recurring errors in production systems with blame-proportional
[11] R. Abreu, P. Zoeteweij, and A. Gemund. 2009. Spectrum-based multiple fault                         logging. In USENIX ATC.
     localization. In ASE. Auckland, NZ, 88–99.                                                    [34] Q. Luo, F. Hariri, L. Eloussi, and D. Marinov. 2014. An empirical analysis of flaky
[12] R. Abreu, P. Zoeteweij, R. Golsteijn, and A. Gemund. 2009. A practical evaluation                  tests. In FSE. Hong Kong, 643–653.
     of spectrum-based fault localization. Journal of Systems and Software 82 (2009),              [35] A. Memon, Z. Gao, B. Nguyen, S. Dhanda, E. Nickell, R. Siemborski, and J. Micco.
     1780–1792.                                                                                         2017. Taming Google-scale continuous testing. In ICSE-SEIP. Buenos Aires,
[13] J. Bell, O. Legunsen, M. Hilton, L. Eloussi, T. Yung, and D. Marinov. 2018. DeFlaker:              Argentina, 233–242.
     Automatically detecting flaky tests. In ICSE. Gothenburg, Sweden, 433–444.                    [36] J. Micco. 2016. Flaky tests at Google and how we mitigate them. https://testing.
[14] A. Bessey, K. Block, B. Chelf, A. Chou, B. Fulton, S. Hallem, C. Henri-Gros, A.                    googleblog.com/2016/05/flaky-tests-at-google-and-how-we.html
     Kamsky, S. McPeak, and D. Engler. 2010. A few billion lines of code later: Using              [37] J. Micco. 2017. The state of continuous integration testing at Google. In ICST,
     static analysis to find bugs in the real world. Commun. ACM 53 (2010), 66–75.                      Keynote. https://bit.ly/2OohAip
[15] M. Christakis and C. Bird. 2016. What developers want and need from program                   [38] K. Muşlu, B. Soran, and J. Wuttke. 2011. Finding bugs by isolating unit tests. In
     analysis: An empirical study. In ASE. Singapore, 332–343.                                          ESEC/FSE. Szeged, Hungary, 496–499.
                                                                                                   [39] M. Naik, A. Aiken, and J. Whaley. 2006. Effective static race detection for Java.
[16] M. Ernst, J. Perkins, P. Guo, S. McCamant, C. Pacheco, M. Tschantz, and C. Xiao.
                                                                                                        In PLDI. Ottawa, Canada, 308–319.
     2007. The Daikon system for dynamic detection of likely invariants. Science of
                                                                                                   [40] M. Naik, C. Park, K. Sen, and D. Gay. 2009. Effective static deadlock detection. In
     Computer Programming 69 (2007), 35–45.
                                                                                                        ICSE. Vancouver, BC, Canada, 386–396.
[17] C. Flanagan and S. Freund. 2008. Atomizer: A dynamic atomicity checker for
                                                                                                   [41] L. Naish, H. Lee, and K. Ramamohanarao. 2011. A model for spectra-based
     multithreaded programs. Science of Computer Programming 71 (2008), 89–109.
                                                                                                        software diagnosis. ACM Transactions on Software Engineering and Methodology
[18] C. Flanagan and S. Freund. 2009. FastTrack: Efficient and precise dynamic race
                                                                                                        20 (2011), 11:1–11:32.
     detection. In PLDI. Dublin, Ireland, 121–133.
                                                                                                   [42] F. Palomba and A. Zaidman. 2017. Does refactoring of test smells induce fixing
[19] M. Fowler. 2011. Eradicating non-determinism in tests. https://bit.ly/2PFHI5B.
                                                                                                        flaky tests?. In ICSME. Shanghai, China, 1–12.
[20] Z. Gao, Y. Liang, M. Cohen, A. Memon, and Z. Wang. 2015. Making system
                                                                                                   [43] M. Rahman and P. Rigby. 2018. The impact of failing, flaky, and high failure tests
     user interactive tests repeatable: When and what should we control?. In ICSE.
                                                                                                        on the number of crash reports associated with Firefox builds. In ESEC/FSE. Lake
     Florence, Italy, 55–65.
                                                                                                        Buena Vista, FL, USA, 857–862.
[21] P. Godefroid. 1997. Model checking for programming languages using VeriSoft.
                                                                                                   [44] A. Santhiar and A. Kanade. 20167. Static deadlock setection for asynchronous C#
     In POPL. Paris, France, 174–186.
                                                                                                        programs. In PLDI. Barcelona, Spain, 292–305.
[22] A. Groce and W. Visser. 2003. What went wrong: Explaining counterexamples. In
                                                                                                   [45] T. Savor, M. Douglas, M. Gentili, L. Williams, K. Beck, and M. Stumm. 2016.
     International SPIN Workshop on Model Checking of Software. Springer, 121–136.
                                                                                                        Continuous deployment at Facebook and OANDA. In ICSE-C. Austin, TX, USA,
[23] J. Ha, J. Yi, P. Dinges, J. Manson, C. Sadowski, and N. Meng. 2013. System to
                                                                                                        21–30.
     uncover root cause of non-deterministic (flaky) tests. In Google Patent. https:
                                                                                                   [46] A. Shi, A. Gyori, O. Legunsen, and D. Marinov. 2016. Detecting assumptions
     //patents.google.com/patent/US9311220
                                                                                                        on deterministic implementations of non-deterministic specifications. In ICST.
[24] M. Harman and P. O’Hearn. 2018. From start-ups to scale-ups: Opportunities
                                                                                                        Chicago, IL USA, 80–90.
     and open problems for static and dynamic program analysis. In SCAM, keynote.
                                                                                                   [47] A. Shi, S. Thummalapenta, S. K Lahiri, N. Bjorner, and J. Czerwonka. 2017. Opti-
     https://bit.ly/2KzBnKQ
                                                                                                        mizing test placement for module-level regression testing. In ICSE. Buenos Aires,
[25] K. Herzig and N. Nagappan. 2015. Empirically detecting false test alarms using
                                                                                                        Argentina, 689–699.
     association rules. In ICSE. Florence, Italy, 39–48.
                                                                                                   [48] W. Sumner, T. Bao, and X. Zhang. 2011. Selecting peers for execution comparison.
[26] M. Hilton, T. Tunnell, K. Huang, D. Marinov, and D. Dig. 2016. Usage, costs, and
                                                                                                        In ISSTA. Toronto, Canada, 309–319.
     benefits of continuous integration in open-source projects. In ASE. Singapore,
                                                                                                   [49] S. Thorve, C. Sreshtha, and N. Meng. 2018. An empirical study of flaky tests in
     426–437.
                                                                                                        Android apps. In ICSME, NIER Track. Madrid, Spain, 534–538.
[27] Y. Jiang, L. Sivalingam, S. Nath, and R. Govindan. 2016. WebPerf: Evaluating
                                                                                                   [50] E. Wong, V. Debroy, R. Gao, and Y. Li. 2014. The DStar method for effective
     what-if scenarios for cloud-hosted web applications. In ACM SIGCOMM.
                                                                                                        software fault localization. IEEE Transactions on Reliability 63 (2014), 290–308.
[28] B. Johnson, Y. Song, E. Murphy-Hill, and R. Bowdidge. 2013. Why don’t software
                                                                                                   [51] S. Zhang, D. Jalali, J. Wuttke, K. Muşlu, W. Lam, M. Ernst, and D. Notkin. 2014.
     developers use static analysis tools to find bugs?. In ICSE. San Francisco, CA,
                                                                                                        Empirically revisiting the test independence assumption. In ISSTA. San Jose, CA,
     USA, 672–681.
                                                                                                        USA, 385–396.




                                                                                             111
