---
url: https://sback.it/publications/msr2017b.pdf
title: To Mock or Not To Mock?
fetched: 2026-06-27
raw: raw.pdf
transport: re-extract
capture_status: ok
---

To Mock or Not To Mock?
            An Empirical Study on Mocking Practices
                       Davide Spadini∗† , Maurício Aniche† , Magiel Bruntink∗ , Alberto Bacchelli†
                                                  ∗ Software Improvement Group

                                                  {d.spadini, m.bruntink}@sig.eu
                                                 † Delft University of Technology

                                          {d.spadini, m.f.aniche, a.bacchelli}@tudelft.nl


   Abstract—When writing automated unit tests, developers often         To support the simulation of dependencies, mocking frame-
deal with software artifacts that have several dependencies. In      works have been developed (e.g., Mockito [7], EasyMock [2],
these cases, one has the possibility of either instantiating the     and JMock [3] for Java, Mock [5] and Mocker [6] for Python),
dependencies or using mock objects to simulate the dependen-
cies’ expected behavior. Even though recent quantitative studies     which provide APIs for creating mock (i.e., simulated) objects,
showed that mock objects are widely used in OSS projects,            setting return values of methods in the mock objects, and
scientific knowledge is still lacking on how and why practitioners   checking interactions between the component under test and
use mocks. Such a knowledge is fundamental to guide further          the mock objects. Past research has reported that software
research on this widespread practice and inform the design of        projects are using mocking frameworks widely [21] [32] and
tools and processes to improve it.
   The objective of this paper is to increase our understanding      has provided initial evidence that using a mock object can ease
of which test dependencies developers (do not) mock and why,         the process of unit testing [29].
as well as what challenges developers face with this practice.          However, empirical knowledge is still lacking on how
To this aim, we create M OCK E XTRACTOR, a tool to mine              and why practitioners use mocks. To scientifically evaluate
the usage of mock objects in testing code and employ it to           mocking and its effects, as well as to help practitioners in
collect data from three OSS projects and one industrial system.
Sampling from this data, we manually analyze how more than           their software testing phase, one has to first understand and
2,000 test dependencies are treated. Subsequently, we discuss        quantify developers’ practices and perspectives. In fact, this
our findings with developers from these systems, identifying         allows both to focus future research on the most relevant
practices, rationales, and challenges. These results are supported   aspects of mocking and on real developers’ needs, as well
by a structured survey with more than 100 professionals. The         as to effectively guide the design of tools and processes.
study reveals that the usage of mocks is highly dependent on
the responsibility and the architectural concern of the class.
                                                                        To fill this gap of knowledge, the goal of this paper is to
Developers report to frequently mock dependencies that make          empirically understand how and why developers apply mock
testing difficult and prefer to not mock classes that encapsulate    objects in their test suites. To this aim, we analyzed more
domain concepts/rules of the system. Among the key challenges,       than 2,000 test dependencies from three OSS projects and
developers report that maintaining the behavior of the mock          one industrial system. We then interviewed developers from
compatible with the behavior of original class is hard and
that mocking increases the coupling between the test and the
                                                                     these systems to understand why some dependencies were
production code.                                                     mocked and others were not. We challenged and supported
                                                                     our findings by surveying 105 developers from software
                      I. I NTRODUCTION                               testing communities. Finally, we discussed our findings with a
   In software testing, it is common that the software artifact      main developer from the most used Java mocking framework.
under test depends on other units [36]. Therefore, when
testing a unit (e.g., a class in object-oriented programming),         The main contributions of this paper are:
developers often need to decide whether to test the unit and         1) A categorization of the most often mocked and not mocked
all its dependencies together (similar to integration testing) or       dependencies, based on a quantitative analysis on three
to simulate these dependencies and test that unit in isolation.         OSS systems and one industrial system (RQ1 ).
   By testing all dependencies together, developers gain real-       2) An empirical understanding of why and when developers
ism: The test will more likely reflect the behavior in produc-          mock, after interviewing developers of analyzed systems
tion [41]. However, some dependencies, such as databases and            and surveying 105 developers (RQ2 ).
web services, may (1) slow the execution of the test [31], (2) be    3) The main challenges faced by developers when making use
costly to properly setup for testing [37], and (3) require testers      of mock objects in the test suites, also extracted from the
to have full control over such external dependencies [18]. By           interviews and surveys (RQ3 ).
simulating its dependencies, developers gain focus: The test         4) An open source tool, namely M OCK E XTRACTOR, that
will cover only the specific unit and the expected interactions         is able to extract the set of mocked and non mocked
with its dependencies; moreover, inefficiencies of testing de-          dependencies in a given Java test suite. The tool is available
pendencies are mitigated.                                               in our on-line appendix [12] and GitHub.
                            II. BACKGROUND                                      database during the test execution). Why do developers mock
           “Once,” said the Mock Turtle at last, with a deep                    the dependency in somes cases and do not mock in other
       sigh, “I was a real Turtle.”                                             cases? Indeed, this is a key question motivating this work.
           — Alice In Wonderland, Lewis Carroll                                    After manually analyzing these tests, we observed that:
                                                                                • In Test 1, the class is concretely instantiated as this test unit
A. Mock objects                                                                   performs an integration test with one of their web services.
   Mock objects are used to replace real software dependencies                    As the test exercises the web service, a database needs to
by simulating their relevant features [28]. Typically, methods                    be active.
of mock objects are designed to return some desired values                      • In Test 2, the class is also concretely instantiated as Is-
given specific input values. Listing II-A shows an example                        sueChangeDao is the class under test.
usage of Mockito, one of the most popular mocking libraries                     • In both Test 3 and Test 4, test units focus on testing two
in Java [32]. We now explain each code block of the example:                      different classes that use IssueChangeDao as part of their
1) At the beginning, one must define the class that should be                     job.
    mocked by Mockito. In our example, LinkedList is being                         This single example shows us that developers may have
    mocked (line 2). The returned object (mockedList) is now                    different reasons to mock or not mock a class. In the remainder
    a mock: It can respond to all existing methods in the                       of this paper, we investigate patterns of how developers mock
    LinkedList class.                                                           by analyzing the use of mocks in software systems and
2) As second step, we provide a new behaviour to the                            we investigate their rationale by interviewing and surveying
    newly instantiated mock. In the example, we inform the                      practitioners on their mocking practices.
    mock to return the string ‘first’ when mockedList.get(0)
    is invoked (line 5) and to throw a RuntimeException on                                     III. R ESEACH M ETHODOLOGY
    mockedList.get(1) (line 7).                                                   The goal of our study is to understand how and why
3) The mock is now ready to be used. In line 10 and 11                          developers apply mock objects in their test suites. To that end,
    the mock will answer method invocations with the values                     we conduct quantitative and qualitative research focusing on
    provided in step 2.                                                         four software systems and address the following questions:
                                                                                RQ1 : What test dependencies do developers mock? When
 1   / / 1 : Mocking L i n k e d L i s t
 2   L i n k e d L i s t mockObj = mock ( L i n k e d L i s t . c l a s s ) ;        writing an automated test for a given class, develop-
 3                                                                                   ers can either mock or use a concrete instance of its
 4 / / 2 : I n s t r u c t i n g t h e mock o b j e c t b e h a v i o u r            dependencies. Different authors [28], [17] affirm that
 5 when ( mockObj . g e t ( 0 ) ) . t h e n R e t u r n ( " f i r s t " ) ;          mock objects can be used when a class depends upon
 6 when ( mockObj . g e t ( 1 ) )
                                                                                     some infrastructure (e.g., file system, caching). We aim
 7    . t h e n T h r o w ( new R u n t i m e E x c e p t i o n ( ) ) ;
 8
                                                                                     to identify what dependencies developers mock by means
 9 / / 3 : I n v o k i n g m e t h o d s i n t h e mock                              of manual analysis in source code from different systems.
10 System . o u t . p r i n t l n ( mockObj . g e t ( 0 ) ) ;                   RQ2 : Why do developers decide to (not) mock specific
11 System . o u t . p r i n t l n ( mockObj . g e t ( 1 ) ) ;                        dependencies? We aim to find an explanation to the
         Listing 1: Example of an object being mocked                                findings in previous RQ. We interview developers from
                                                                                     the analyzed systems and ask for an explanation on why
   Overall, whenever developers do not want to rely on the real                      some dependencies are mocked while others are not.
implementation of a dependency, (e.g., to isolate a unit test)                       Furthermore, we survey software developers with the goal
they can simulate it and define the expected behavior using                          of challenging the findings from the interviews.
the aforementioned approach.                                                    RQ3 : Which are the main challenges experienced with
                                                                                     testing using mocks? Understanding challenges sheds
B. Motivating example
                                                                                     a light on important aspects on which researchers and
   Sonarqube is a popular open source system that provides                           practitioners can effectively focus next. Therefore, we
continuous code inspection [10]. In January of 2017, Sonar-                          investigate the main challenges developers face using
qube contained over 5,500 classes, 700k lines of code, and                           mocks by means of interviews and surveys.
2,034 test units. Among all test units, 652 make use of mock
objects, mocking a total of 1,411 unique dependencies.                          A. Sample selection
   Let us consider the class IssueChangeDao as an example.                         We focus on projects that routinely use mock objects. We
This class is responsible for accessing the database regarding                  analyze projects that make use of Mockito, the most popular
changes in issues (changes and issues are business entities of                  framework in Java with OSS projects [32].
the system). To that end, this class uses MyBatis [8], a Java                      We select three open source software projects (i.e., Sonar-
library for accessing databases.                                                qube [10], Spring [11], VRaptor [13]) and a software system
   There are four test units that use IssueChangeDao. The                       from an industrial organization we previously collaborated
dependency is mocked in two of them; in the other two, the                      with (Alura [1]); Table I details their size. In the following,
test creates a concrete instance of the database (to access the                 we describe their suitability to our investigation.
                                         TABLE I: Description of our studied sample (N=4).

                             # of                      # of      # of test               # of           # of   Sample size      Sample size
   Project                               LOC
                          classes                       test        units             mocked     not mocked     of mocked    of not mocked
                                                      units    with mock         dependencies   dependencies    (CL=95%)         (CL=95%)
   Sonarqube               5,771         701k        2,034            652              1,411         12,136           302              372
   Spring Framework         6561         997k        2,020            299                670         21,098           244              377
   VRaptor                   551          45k          126             80                258          1,075           155              283
   Alura                   1,009          75k          239             91                229          1,436           143              302
   Total                  13.892       1.818k        4.419          1.122              2,568         35,745           844            1,334

Spring Framework. Spring provides an extensive infrastruc-                      the test method. Every time one of the two options is found
tural support for Java developers; its core serves as a base for                in the code, we identify the type of the class that is mocked.
many other offered services, such as dependency injection and                   The class is then marked as ‘mocked’ in that test unit. If
transaction management.                                                         a dependency appears more than once in the test unit, we
Sonarqube. Sonarqube is a quality management platform that                      consider it ‘mocked’. A dependency may be considered
continuously measures the quality of source code and delivers                   ‘mocked’ in one test unit, but ‘not mocked’ in another.
reports to its developers.                                                   4) We mark dependencies as ‘not mocked’ by subtracting the
VRaptor. VRaptor is an MVC framework that provides an                           mocked dependencies from the set of all dependencies.
easy way to integrate Java EE capabilities (such as CDI) and                    2. Manual analysis. To answer what test dependencies de-
to develop REST webservices.                                                 velopers mock, we analyzed the previously extracted mocked
Alura. Alura is a proprietary web e-learning system used by                  and non mocked dependencies. The goal of the analysis is to
thousands of students and teachers in Brazil; it is a database-              understand the main concern of the class in the architecture of
centric system developed in Java.                                            the software system (e.g., a class is responsible for represent-
                                                                             ing a business entity, or a class is responsible for persisting
B. Data Collection and Analysis
                                                                             into the database). Defining the architectural concern of a
   The research method we used to answer our research ques-                  class is not an easy task to be automated, since it is context-
tions follows a mixed qualitative and quantitative approach,                 specific [12], thus we decided to perform a manual analysis.
which we depict in Figure 1: (1) We automatically collected                  The first two authors of the paper conducted this analysis after
all mocked and non-mocked dependencies in the test units                     having studied the architecture of the four systems.
of the analyzed systems, (2) we manually analyzed a sample                      Due to the size of the total number of mocked and non
of these dependencies with the goal of understanding their                   mocked dependencies (~38,000), we analyzed a random sam-
architectural concerns as well as their implementation, (3) we               ple. The sample is created with the confidence level of 95%
grouped these architectural concerns into categories, which                  and the error (E) of 5%, i.e., if in the sample a specific
enabled us to compare mocked and non mocked dependencies                     dependency is mocked f% of the times, we are 95% confident
among these categories, (4) we interviewed developers from                   that it will be mocked f % ± 5% in the entire test suite. Since
the studied systems to understand our findings, and (5) we                   projects belong to different areas and results can be completely
enhanced our results in a on-line survey with 105 respondents.               different from each other, we created a sample for each project.
   1. Data collection. To obtain data on mocking practices,                  We produced four samples, one belonging to each project. This
we first collected all the dependencies in the test units of                 gave us fine-grained information to investigate mock practices
our systems performing static analysis on their test code. To                within each project.
this aim, we created the tool M OCK E XTRACTOR [38], which                      In Table I we show the final number of analyzed dependen-
implements the algorithm below:                                              cies (844 + 1, 334 = 2, 178 dependencies).
1) We detect all test classes in the software system. As done                   The manual analysis procedure was as follows:
    in past literature (e.g., Zaidman et al. [43]), we consider a            • Each researcher was in charge of two projects. The selection
    class to be a test when its name ends with ‘Test’ or ‘Tests’.              was done by convenience: The second author was already
2) For each test class, we extract the (possibly extensive) list               familiar with the internal structure of VRaptor and Alura.
    of all its dependencies. Examples of dependencies are the                • All dependencies in the sample were listed in a spreadsheet
    class under test itself, its required dependencies, and utility            in which both researchers had access. Each row contained
    classes (e.g., lists and test helpers).                                    information about the test unit that dependency was found
3) We mark each dependency as ‘mocked’ or ‘not mocked’.                        and a boolean indicating if that dependency was mocked.
    Mockito provides two APIs for creating a mock from a                     • For each dependency in the sample, the researcher manually
    given class:1 (1) By making use of the @Mock annotation in                 inspected the source code of the class. To fully understand
    a class field or (2) by invoking Mockito.mock() inside                     the class’ architectural concern, researchers were allowed to
   1 Mockito can also generate spies which are out of the scope of
                                                                               navigate through any other relevant piece of code.
this paper. More information can be found at Mockito’s documentation:        • After understanding the concern of that class, the researcher
http://bit.ly/2kjtEi6.                                                         filled the “Category” column with what best describes the
Data collection                                                                     Data analysis
 Sonarqube                                 1                                                          Manual analysis                                               Categorisation

           T   A


                                   T
                                   A              P   1
                                                                     T
                                                                     A       P 1                                                                        Discussion
                                                                     …       …                                                    Commit
                                                                                                                                   Commit
                                                                                                                                      132
                                                                                                                                    Commit
                                           Mock                                                                                  Comments
                                                                                                                                  Comments
                                                                                                                                   Comments
           P   1
                                   P   3          P
                                                  2                  T
                                                                     n       P n                                                  categories
                              13,547 dependencies                674 dependencies

 Alura                                                                                                                              3
                                                                                         Sample 1                                                                Commit
                                                                                                                                                                    7
                                                                                                                                                                Comments
           T   A
                                                                                                        Manual analysis
                                   T
                                   A              P   1
                                                                     T
                                                                     A       P 1
                                                                                                         (2nd author)
                                                                     …       …                                                                                 Categories
                                           Mock
           P                       P   3          P
                                                  2                  T
                                                                     n       P n
               1
                                                                                                                                                                        Interviews &
                               1,665 dependencies                445 dependencies                                                                                         Validation
                                                                                         Validation       2
 VRaptor                                                                                                                                                            4
                                                                                                                                   Interview
           TA
                                                                                                                                   Guideline
                                   T   A          P   1
                                                                     T
                                                                     A       P 1

                                                                     …       …
                                           Mock                                                                                                3 developers
           P   1
                                   P   3          P   2              T
                                                                     n       P n                                                                                         Commit
                                                                                                                                                                          Review
                                                                                                                                                                           Commit
                                                                                                                                                                        Comments
                                                                                                                                                                            Interview
                                                                                                                                                                         Comment
                               1,333 dependencies                438 dependencies                                                                                         Comments
                                                                                         Sample 2                                                                          Transcript


 Spring Framework
                                                                                                      Manual analysis
                                                                                                                             5
                                                                                                       (1st author)
           TA                                                                                                                 Commit
                                                                                                                               Commit
                                   T   A          P   1
                                                                     T
                                                                     A       P 1                                             Comments
                                                                                                                              Comments
                                                                     …       …
                                           Mock
                                   P   3          P   2              T
                                                                     n       P n
           P   1


                              21,768 dependencies                621 dependencies                                               Survey           Affinity Diagram

                   MockExtractor                          Sampling                                                        (105 respondents)

                                                          Fig. 1: The mixed approach research method applied.


  concern. No categories were defined up-front. In case of                                                    TABLE II: Profile of the interviewees
  doubt, the researcher first read the test unit code; if not                                                                                         Years of programming
  enough, he then talked with the other research.                                             Project                   ID   Role in the project
                                                                                                                                                                  experience
• At the end of each day, the researchers discussed together                                  Spring Framework        D1     Lead Developer                                        25
  their main findings and some specific cases.                                                VRaptor                 D2     Developer                                             10
                                                                                              Alura                   D3     Lead Developer                                         5
   The full process took seven full days. The total number of
categories was 116. We then started the second phase of the                                    4. Interviews. We used results from the previous RQ as an
manual analysis, focused on merging categories.                                             input to the data collection procedure of RQ2 . We designed an
   3. Categorization. To group similar categories we used                                   interview in which the goal was to understand why developers
a technique similar to card sort [35]: (1) each category                                    did mock some roles and did not mock other roles. The
represented a card, (2) the first two authors analyzed the cards                            interview was semi-structured and was conducted by the first
applying open (i.e., without predefined groups) card sort, (3)                              two authors of this paper. For each finding in previous RQ, we
the researcher who created the category explained the reasons                               made sure that the interviewee described why they do (or do
behind it and discussed a possible generalization (making the                               not) mock that particular category, what the perceived advan-
discussion more concrete by showing the source code of the                                  tages and disadvantages are, and any exceptions to this rule.
class was allowed during the discussion), (4) similar categories                            Our full interview protocol is available in the appendix [12].
were then grouped into a final, higher level category. (5) at the                              We conducted 3 interviews with active, prolific developers
end the authors gave a name to each final category.                                         from 3 projects (unfortunately no developer from Sonarqube
   After following this procedure for all the 116 categories, we                            was available for an interview). Table II shows the intervie-
obtained a total of 7 categories that describe the concerns of                              wees’ details.
classes.                                                                                       We started each interview by asking general questions about
   The large difference between 116 and 7 is the result of most                             mocking practices. More specifically, we were interested in
concerns being grouped into two categories: ‘Domain object’                                 understanding why and what classes they commonly mock.
and ‘External dependencies’. The former classes always repre-                               Afterwards, we focused on the results gathered by answering
sented some business logic of the system and had no external                                the previous RQ. We presented the interviewee with two
dependencies. The full list of the 116 categories is available                              tables: one containing the results of all projects (Figure 2) and
in our on-line appendix [12].                                                               another one containing only the results of the interviewee’s
project. For each category, we presented the findings and          15 and 15% have more than 15 years of experience. The
solicit an interpretation (e.g., by explaining why it happens      most used programming language is Java (24%), the second is
in their specific project and by comparing with what we saw        JavaScript (19%) and the third one is C# (18%). The mocking
in other projects). From a high-level perspective, we asked:       framework most used by the respondents is Mockito (33%)
1) Can you explain this difference? Please, think about your       followed by Moq (19%) and Powermock (5%).
    experience with this project in particular.
                                                                   C. Threats to Validity
2) We observe that your numbers are different when compared
    to other projects. In your opinion, why does it happen?           Our methodology may pose some threats to the validity of
3) In your experience, when should one mock a <category>?          the results we report in Section IV. We discuss them here.
    Why?                                                              1) Construct validity: Threats to construct validity concern
4) In your experience, when should one not mock a <cate-           our research instruments. We develop and use M OCK E X -
    gory>? Why?                                                    TRACTOR to collect dependencies that are mocked in a test
5) Are there exceptions?                                           unit by means of static code analysis. As with any static
6) Do you know if your rules are also followed by the other        code analysis tool, M OCK E XTRACTOR is not able to capture
    developers in your project?                                    dynamic behavior (e.g., mock instances that are generated in
   Throughout the interview, one of the researchers was in         helper classes and passed to the test unit). In these cases,
charge of summarizing the answers. Before finalizing the           the dependency would have been considered “non mocked”.
interview, we revisited the answers with the interviewee to        We mitigate this issue by (1) making use of a large random
validate our interpretation of their opinions. Finally, we asked   samples in our manual analysis, and (2) manually inspecting
questions about challenges with mocking.                           the results of M OCK E XTRACTOR in 100 test units, in which
   Interviews were conducted via Skype and fully recorded.         we observed that such cases never occurred, thus giving us
Each of them was manually transcribed by the researchers.          confidence regarding the reliability of our data set.
With the full transcriptions, we performed card sorting [39],         As each class is manually analyzed by only a single
[20] to identify the main themes.                                  researcher and there could be divergent opinions despite
   As a complement to the research question, whenever feasi-       the aforementioned discussion, we measured their agreement.
ble, we also validated interviewees’ perceptions by measuring      Each researcher analyzed 25 instances that were made by
them in their own software system.                                 the other researcher in both of his two projects, totaling 100
   5. Survey. To challenge and expand the concepts that            validated instances as seen in Figure 1, Point 2. The final
emerged during the previous phases, we conducted a survey.         agreement on the 7 categories was 89%.
All questions were derived from the results of previous RQs.          2) Internal validity: Threats to internal validity concern
The survey had four main parts. In the first part, we asked        factors we did not consider that could affect the variables and
participants about their experience in software development        the relations being investigated. In our study, we interview de-
and mocking. The second part of the survey asked participants      velopers from the studied software to understand why certain
about how often they make use of mock objects in each of           dependencies are mocked and not mocked. Clearly, a single
the categories found during the manual analysis. The third         developer does not know all the implementation decisions in
part asked participants about how often they mock classes in       a software system. We mitigate this issue by (1) showing the
specific situations, such as when the class is too complex or      data collected in RQ1 first and (2) not asking questions about
coupled. The fourth part was focused on asking participants        the overall categories that we manually coined in RQ1.
about challenges with mocking. Except for this last question,         In addition, their opinions may also be influenced by other
which was open-ended and optional, all the other questions         factors, such as current literature on mocking (which could
were closed-ended and participants had to choose between a         may have led them to social desirability bias [33]) or other
5-point Likert scale.                                              projects that they participate in. To mitigate this issue, we
   The survey was initially designed in English. We compiled       constantly reminded interviewees that we were discussing the
Brazilian Portuguese translation, to reach a broader, more         mocking practices specifically of their project. At the end of
diverse population. Before deploying the survey, we first          the interview, we asked them to freely talk about their ideas
performed a pilot of both versions with four participants;         on mocking in general.
we improved our survey based on their feedbacks (changes              3) External validity: Threats to external validity concern
were all related to phrasing). We then shared our survey via       the generalization of results. Our sample contains four Java
Twitter (authors tweeted in their respective accounts), among      systems (one of them closed source), which is small compared
our contacts, and in developers’ mailing lists. The survey         to the overall population of software systems that make use
ran for one week. We analyzed the open questions by also           of mocking. We reduce this issue by collecting the opinion of
performing card sorting. The full survey can be found in our       105 developers from a variety of projects about our findings.
on-line appendix [12].                                             Further research in different projects in different programming
   We received a total of 105 answers from both Brazilian          languages should be conducted.
Portuguese and English surveys. 21% of the respondents have           Furthermore, we do not know the nature of the population
between 1 and 5 years of experience, 64% between 6 and             that responded to our survey, hence it might suffer from a self-
selection bias. We cannot calculate the response rate of our
                                                                               DATABASE                                          72%(167)         28%(64)
survey; however, from the responses we see a general diversity
in terms of software development experience that appears to
match in our target population.                                             WEB SERVICE                                        69%(182) 31%(82)


                         IV. R ESULTS
                                                                              EXTERNAL
                                                                                                                               68%(140) 32%(67)
  In this section, we present the results to our research                 DEPENDENCIES

questions aimed at understanding how and why developers
apply mock objects in their test suites, as well as which                 DOMAIN OBJECT                    36%(319) 64%(579)

challenges they face in this context.
                                                                                           7%
RQ1. What test dependencies do developers mock?                           JAVA LIBRARIES   (12)
                                                                                                93%(160)


   As we show in Table I, we analyzed 4,419 test units of
which 1,122 (25.39%) contain at least one mock object. From                TEST SUPPORT    6% 94%(358)

the 38,313 collected dependencies from all test units, 35,745
                                                                      Percentage of mocked dependencies           Percentage of non-mocked dependencies
(93.29%) are not mocked while 2,568 (6.71%) are mocked.
   As the same dependency may appear more than once in
                                                                      Fig. 2: How often each architectural role is mocked and not
our dataset (i.e., , a class can appear in multiple test units), we
                                                                               mocked in analyzed systems (N = 2, 178)
calculated the unique dependencies in our dataset. We obtained
a total of 11,824 not mocked and 938 mocked dependencies.             •  Unresolved: Dependencies that we were not able to solve.
Interestingly, the intersection of these two sets reveals that 650       For example, classes belonging to a sub-module of the
dependencies (70% of all dependencies mocked at least once)              project which the source code is not available.
were both mocked and not mocked in the test suite.                       Numbers are quite similar when we look at each project
   In Figure 2, we show how often each role is mocked in              separately. Exceptions are for databases (Alura and Sonarqube
our sample in each of the seven categories found during our           mock ~60% of databases dependencies, Spring mocks 94%)
manual analysis. One may note that “databases” and “web               and domain objects (while other projects mock them ~30% of
services” can also fit in the “external dependency” category;         times, Sonarqube mocks 47%). We present the numbers for
we separate these two categories as they appeared more                each project in our online appendix [12].
frequently than other types of external dependencies.                    We observe that Web Services and Databases are the most
   In the following, we explain each category:                        mocked dependencies. On the other hand, there is no clear
• Domain object: Classes that contain the (business) rules            trend in Domain objects: numbers show that 36% of them
   of the system. Most of these classes usually depend on             are mocked. Even though the findings are aligned with the
   other domain objects. They do not depend on any external           technical literature [28], [23], further investigation is necessary
   resources. The definition of this category fits well to the        to understand the real rationale behind the results.
   definition of Domain Object [15] and Domain Logic [16]                In contrast Test support and Java libraries are almost
   architectural layers. Examples are entities, services and          never mocked. The former is unsurprising since the category
   utility classes.                                                   includes fake classes or classes that are created to support the
• Database: Classes that interact with an external database.          test itself.
   These classes can be either an external library (such as
   Java SQL, JDBC, Hibernate, or ElasticSearch APIs) or
                                                                           RQ1 . Classes that deal with external resources, such as
   a class that depends on such external libraries (e.g., an
                                                                           databases and web services are often mocked. Interest-
   implementation of the Data Access Object [16] pattern).
                                                                           ingly, there is no clear trend on mocking domain objects.
• Native Java libraries: Libraries that are part of the Java
   itself. Examples are classes from Java I/O and Java Util
   classes (Date, Calendar).                                          RQ2. Why do developers decide to (not) mock specific depen-
• Web Service: Classes that perform some HTTP action. As              dencies?
   with the database category, this dependency can be either an
                                                                        In this section, we summarize the answers obtained during
   external library (such as Java HTTP) or a class that depends
                                                                      our interviews and surveys. We refer to the interviewees by
   on such library.
                                                                      their ID in Table II.
• External dependency: Libraries (or classes that make use
   of libraries) that are external to the current project. Examples   Mocks are often used when the concrete implementation is
   are Jetty and Ruby runtimes, JSON parsing libraries (such          not simple. All interviewees agree that certain dependencies
   as GSON), e-mail libraries, etc.                                   are easier to mock than to use their concrete implementation.
• Test support: Classes that support testing itself. Examples         They mentioned that classes that are highly coupled, complex
   are fake domain objects, test data builders and web services       to set up, contain complex code, perform a slow task, or
   for tests.                                                         depend on external resources (e.g., databases, web services
or external libraries) are candidates to be mocked. D2 gives a        so this is a green light for me to know that I don’t need to
concrete example: “It is simpler to set up a in-memory list with      test B again.” All interviewees also mention that the same rule
elements than inserting data into the database.” Interviewees         applies if the domain object is highly coupled.
affirmed that whenever they can completely control the input            Figure 4 shows that answers about mocking Domain objects
and output of a class, they prefer to instantiate the concrete        vary. Interestingly, there is a slight trend towards not mocking
implementation of the class rather than mocking it. As D1             them, in line to our findings during the interviews and in RQ1.
stated: “if given an input [the production class] will always
return a single output, we do not mock it.”                           Native Java objects and libraries are usually not mocked.
   In Figure 3, we see that survey respondents also often mock        According to D1, native Java objects are data holders
dependencies with such characteristics: 48% of respondents            (e.g., String and List) that are easy to instantiate with the
said they always or almost always mock classes that are               desired value. Thus no need for mocking. D1 points out
highly coupled, and 45.5% when the class difficult to set up.         that some native classes cannot even be mocked as they
Contrarily to our interviewees, survey respondents report to          can be final (e.g., String). D2 discussed the question from a
mock less often when it comes to slow or complex classes              different perspective. According to him, developers can trust
(50.4% and 34.5% of respondents affirm to never or almost             the provided libraries, even though they are “external,” thus,
never mock in such situations, respectively).                         there is no need for mocking. Both D1 and D2 made an
                                                                      exception for the Java I/O library: According to them, dealing
Mocks are not used when the focus of the test is the                  with files can also be complex, and thus, they prefer to mock.
integration. Interviewees explained that they do not use              D3, on the other hand, affirms that in their software, they
mocks when they want to test the integration with an external         commonly do not mock I/O as they favor integration testing.
dependency itself, (e.g., a class that integrates with a database).
In these cases they prefer to perform a real interaction between         These findings match our data from RQ1, where we see that
the unit under test and the external dependency. D1 said “if          Native Java Libraries are almost never mocked. Respondents
we mock [the integration], then we wouldn’t know if it actually       also had a similar perception: 82% of them affirm to never or
works. [...] I do not mock when I want to test the database           almost never mock such dependencies.
itself; I wanna make sure that my SQL works. Other than that,         Database, web services, and external dependencies are
we mock.” This is also confirmed in our survey (Figure 3), as         slow, complex to set up, and are good candidates to
our respondents also almost never mock the class under test.          be mocked. According to the interviewees, that is why
   The opposite scenario is when developers want to unit test a       mocks should be applied in such dependencies. D2 said:
class that depends on a class that deals with external resources,     “Our database integration tests take 40 minutes to execute,
(e.g., Foo depends on Boo, and Boo interacts with a database).        it is too much”. These reasons also matches with technical
In this case, developers want to test a single unit without           literature [28], [23].
the influence of the external dependencies, thus developers
                                                                         All participants have a similar opinion when it comes to
evaluate whether they should mock that dependency. D2 said:
                                                                      other kinds of external dependencies/libraries, such as CDI or
“in unit testing, when the unit I wanna test uses classes that
                                                                      a serialization library: When the focus of the testing is the
integrate with the external environment, we do not want to test
                                                                      integration itself, they do not mock. Otherwise, they mock.
if the integration works, but if our current unit works, [...] so
                                                                      D2 said: “When using CDI [Java’s Contexts and Dependency
we mock the dependencies.”
                                                                      Injection API], it is really hard to create a concrete [CDI]
Interfaces are mocked rather than one of their specific               event: in this case we usually prefer to mock it”. Two
implementations. Interviewees agree that they often mock              interviewees (D1 and D2) affirmed that libraries commonly
interfaces. They explain that an interface can have several           have extensive test suites, thus developers do not need to “re-
implementations and they prefer to use a mock to not rely             test”. D3 had a different opinion: Developers should re-test
on a specific one. D1 said: “when I test operations with side         the library as they cannot always be trusted.
effects [sending an email, doing a HTTP Request] I create an             In Figure 4, we observe that respondents always or almost
interface that represents the side effect and [instead of using       always mock Web services (~82%), External dependencies
a specific implementation] I mock directly the interface.”            (~79%) and Databases (~71%). This result confirm the pre-
Domain objects are usually not mocked. According to                   vious discovery that when developers do not want to test the
the interviewees, domain objects are often plain old Java             integration itself, they prefer to mock these dependencies.
objects, commonly composed by a set of attributes, getters and
setters. These classes also commonly do not deal with external          RQ2 . The architectural role of the class is not the
resources. Thus, these classes tend to be easily instantiated and       only factor developers take into account when mocking.
set up. However, if a domain object is complex (i.e., contains          Respondents report to mock when to use the concrete
complicated business logic or not easy to set up), developers           implementation would be not simple, e.g., the class would
may mock them. Interviewee D2 says: “[if class A depends                be too slow or complex to set up.
on the domain object B] I’d probably have a BTest testing B
           The class was not
                                                        7 6   10    24                       55                         Web service                           6   7     23            45
           the unit under test

          The class was very
                                              13        14    27        32              18                    External dependency                           5 9   4     24                44
   coupled with other classes

          The class was very
                                              12        16    28        29             18
            diﬃcult to set up                                                                                              Database                     9    11   6       26              40

          The class was very
                                         15         21        32    23            13
               very complex                                                                                        Domain object                13      27        10   17      16
        The class would have
                                  20               31         20   15        15
        been too slow to test
                                                                                                                        Native library     38           27        4 5 6


         Never     Almost never        Occasionally/ Sometimes     Almost always                  Always        Never       Almost never   Occasionally/ Sometimes        Almost always        Always

         Fig. 3: Reasons to use mock objects (N = 105)                                                     Fig. 4: Frequency of mocking objects per category (N = 105)

RQ3. Which are the main challenges experienced with testing                                                 (which are not mockable by default), file uploads in PHP,
using mocks?                                                                                                interfaces in dynamic languages, and the LINQ language
   We summarize the main challenges that appeared in the                                                    feature in C#.
interviews and in the answers of our question about challenges                                              The relationship between mocks and good quality code.
in the survey (which we received 61 answers). Categories                                                    Mocks may reduce test readability and be difficult to maintain.
below represent the main themes that emerged during card                                                    Survey respondents state that the excessive use of mocks is
sorting.                                                                                                    an indicative of poorly engineered code. Surprisingly during
Dealing with coupling. Mocking practices deal with different                                                the interviews, D1, D2 and D3 mentioned the same example
coupling issues. On one hand, the usage of mocks in test in-                                                where using mocks can hide a deeper problem in the system’s
creases the coupling between the test and the production code.                                              design: “when you have to test class A, and you notice that
On the other hand, the coupling among production classes                                                    it has 10/15 dependencies, you can mock it. However, you are
themselves can also be challenging for mocking. According                                                   hiding a problem: a class with 15 dependencies is probably
to a participant, “if code has not been written with proper                                                 a smell in the code.” In this scenario they find it much easier
decoupling and dependency isolation, then mocking is difficult                                              to mock the dependency as it is highly coupled and complex.
(if not impossible).” This matches with another participant’s                                               However, they say this is a symptom of a badly designed class.
opinions who mentions to not have challenges anymore, by                                                    D3 added: “good [production] code ease the process of testing.
having “learned how to separate concepts.”                                                                  If the [production] code structure is well defined, we should
                                                                                                            use less mocks”. Interviewee D3 also said “I always try to use
Getting started with mocks. Mocks can still be a new                                                        as less mocks as possible, since in my opinion they hide the
concept for many developers. Hence, its usage may require                                                   real problem. Furthermore, I do not remember a single case
experienced developers to teach junior developers (which,                                                   in which I found a bug using mocks’. A survey respondent
according to another participant, usually tend to mock too                                                  also shares the point that the use of mocks does not guarantee
much). In particular, a participant said that mock objects are                                              that your code will behave as expected in production: “You
currently a new concept for him/her, and thus, s/he is having                                               are always guessing that what you mock will work (and keep
some trouble understanding it.                                                                              working) that way when using the real objects.”
Mocking in legacy systems. Legacy systems can pose some                                                     Unstable dependencies. A problem when using mocks is
challenges for users of mocks. According to a respondent,                                                   maintaining the behavior of the mock compatible with the
testing a single unit in such systems may require too much                                                  behavior of original class, especially when the class is poorly
mocking (“to mock almost the entire system”). Another par-                                                  designed or highly coupled. As the production class tends to
ticipant even mentions the need of using PowerMock [9]                                                      change often, the mock object becomes unstable and, as a
(a framework that enables Java developers to mock certain                                                   consequence, more prone to change.
classes that might be not possible without bytecode manip-
ulation, e.g., final classes and static methods) in cases where
the class under test is not designed for testability. On the other                                            RQ3 . The use of mocks poses several challenges. Among
hand, mocking may be the only way to perform unit testing in                                                  all, a major problem is maintaining the behavior of the
such systems. According to a participant: “in legacy systems,                                                 mock compatible with the original class. Furthermore,
where the architecture is not well-decoupled, mocking is the                                                  mocks may hide important design problems. Finally, while
only way to perform some testing.”                                                                            mocking may be the only way to test legacy systems, using
                                                                                                              them in such systems is not a straightforward task.
Non-testable/Hard-to-test classes. Some technical details
may impede the usage of mock objects. Besides the lack of
design by testability, participants provide different examples                                                                              V. D ISCUSSION
of implementation details that can interfere with mocking.                                                    In this section we discuss the main findings and their
Respondents mentioned the use of static methods in Java                                                     implications for both practitioners and future research. We
also present the results of a debate about our findings with        mocking process of any of the analyzed categories (Figure 2),
a main developer from Mockito. Finally, we provide an initial       he stated: “If someone tells us that s/he is spending 100 boiler-
discussion on quantitatively mining mocking practices.              plate lines of code to mock a dependency, we can provide a
                                                                    better way to do it. [...] But for now, I can not see how to
A. Empirical evidence on mocking practices                          provide specific features for databases and web services, as
   Mocking is a popular topic among software developers. Due        Mockito only sees the interface of the class, and not its internal
to its importance, different authors have been writing technical    behavior.”
literature on mock objects (e.g., [19], [31], [18], [34], [24],        After, we focused on the challenges, as we conjecture that
[27]), ranging from how to get started with mocks to best           it is the most important and useful part for practitioners and
practices. Our research complements such technical literature       future research and that his experience can shed a light on
in three ways that we discuss below.                                them. D4 agreed with all the challenges specified by our
   First, we provide concrete evidence on which of the existing     respondents. When discussing how Mockito could help devel-
practices in technical literature developers actually apply. For    opers with all the coupling challenges (unstable dependencies,
example, Meszaros [31] suggests that components that make           highly coupled classes), he affirmed that the tool itself can not
testing difficult are candidates to be mocked. Our research         help and that the issue should be fixed in the production class:
confirms it by showing that developers also believe these           “When a developer has to mock a lot of dependencies just to
dependencies should be mocked (RQ2) and that, in practice,          test a single unit, he can do it! However, it is a big red flag
developers do mock them (RQ1).                                      that the unit under test is not well designed.”. This reinforces
   Second, by providing a deeper investigation on how and           the relationship between the excessive use of mocks and code
why developers use mock objects. As a side effect, we also          quality.
notice how the use of mock objects can drive the developer’s           When we discussed with him about a possible support for
testing strategy. For instance, mocking an interface rather         legacy systems in Mockito, D4 said Mockito developers have
than using one concrete implementation makes the test to            a philosophical debate internally: They want to keep a clear
become “independent of a specific implementation”, as the test      line of what this framework should and should not do. Non
exercises the abstract behavior that is offered by the interface.   supported features such as the possibility of mocking a static
Without the usage of a mock, developers would have to choose        method would enable developers to test their legacy code more
one out of the many possible implementations of the interface,      easily. However, he stated: “I think the problem is not adding
making the test more coupled to the specific implementation.        this feature to Mockito, probably it will require just a week
The use of mock objects can also drive developers towards a         of work, the problem is: should we really do it? If we do it,
better design: Our findings show that a class that requires too     we allow developers to write bad code.” He also said that
much mocking could have been better designed to avoid that.         final classes can be mocked in Mockito 2.0; interestingly, the
Interestingly, the idea of using the feedback of the test code      feature was not motivated by a willingness to ease the testing
to improve the quality of production code is popular among          of legacy systems, but by developers using Kotlin language [4],
TDD practitioners [14].                                             in which every class is final by default.
   Third, by providing a list of challenges that can be tackled        To face the challenge of getting started with mocks, D4
by researchers, practitioners, and tool makers. Most challenges     mentioned that Mockito documentation is already extensive
faced by developers are purely technical, such as applying          and provides several examples on how to better use the
mocks in legacy systems and in poorly-designed classes, or          framework. However, according to him, knowing what should
even dealing with unstable production classes. Interestingly,       be mocked and what should not be mocked comes with
none of the participants complained about the framework itself      experience.
(e.g., missing features or bugs).                                   C. Quantitatively mining mocking practices
B. Discussing with a developer from Mockito                            Our study sheds lights on some of the most used practices
                                                                    of mocking objects for testing and their reasons. Work can
   To get an even deeper understanding of our results and
                                                                    be done to check and generalize some of the answers given
challenge our conclusions, we interviewed a developer from
                                                                    by developers by means of software data mining. This would
Mockito, showing him the findings and discussing the chal-
                                                                    have the advantage of a more objective view and quick
lenges. We refer to him as D4.
                                                                    generalizability to other systems. We take a first step into
   D4 agreed on the findings regarding what developers should
                                                                    this direction by conducting an initial analysis to test the
mock: According to him, databases and external dependencies
                                                                    water and see whether some of our qualitative findings can
should be mocked when developers do not test the integration
                                                                    be confirmed/denied by means of software data mining. In
itself, while Java libraries and data holders classes should
                                                                    the following paragraphs, we discuss the results of our initial
never be mocked instead. Furthermore, D4 also approved what
                                                                    analysis and we provide possible alternatives to mine this
we discovered regarding mocking practices. He affirmed that a
                                                                    information from code repositories.
good practice is to mock interfaces instead of real classes and
that developers should not mock the unit under test. When we        The unit under test is never mocked. To confirm this
argued whether Mockito could provide a feature to ease the          assertion, we automatically analyzed all test classes. For each
test unit, we verified whether the unit under test (e.g. class A                       VI. R ELATED W ORK
in the test unit ATest) has been mocked or not. Results show          Despite the widespread usage of mocks, very few studies
that over ~38,000 analyzed dependencies the unit under test        analyzed current mocking practices. Mostafa et al. [32] con-
is never mocked in any of the projects.                            ducted an empirical study on more than 5,000 open source
                                                                   software projects from GitHub, analyzing how many projects
Unless it is the unit under test, database dependencies are
                                                                   are using a mocking framework and which Java APIs are the
always mocked. To confirm this assumption, for each database
                                                                   most mocked ones. The result of this study shows that 23%
dependency (information retrieved from our previous manual
                                                                   of the projects are using at least one mocking framework and
analysis in RQ1 ) outside its own test, we counted the number
                                                                   that Mockito is the most widely used (70%).
of times in which the dependency was not mocked. In case
                                                                      Marri et al. [29] investigated the benefits of using mock
of Alura, we found that 90% of database dependencies are
                                                                   objects. The study identifies the following two benefits: 1)
mocked when not in their specific test unit. When extending
                                                                   mock objects enable unit testing of the code that interacts with
this result to all the projects, we obtain an average of 81%.
                                                                   external APIs related to the environment such as a file system,
Complex and coupled classes should be mocked. We take              and 2) enable the generation of high-covering unit tests.
into account two metrics: CBO (Coupling between objects)              Taneja et al. [40] stated that automatic techniques to gen-
and McCabe’s complexity [30]. We choose these metrics              erate tests face two significant challenges when applied to
since they have been widely discussed during the interviews.       database applications: (1) they assume that the database that
Furthermore, as pointed out during the surveys, developers         the application under test interacts with is accessible, and
mock when classes are very coupled or difficult to set up.         (2) they usually cannot create necessary database states as a
                                                                   part of the generated tests. For this reasons they proposed
   With the metrics value for each production class in the four    an “Automated Test Generation” for Database Applications
systems, we compare the values from classes that are mocked        using mock objects, demonstrating that with this technique
with the values from classes that are not mocked. In general,      they could achieve better test coverage.
as a class can be mocked and not mocked multiple times,               Karlesky et al. [25] applied Test-Driven Development and
we apply a simple heuristic to decide in which category it         Continuous Integration using mock objects to embedded soft-
should belong: If the class has been mocked more than 50%          wares, obtaining an order of magnitude or more reduction in
of the times, we put it in the ‘mocked’ category, and vice-versa   software flaws, predictable progress, and measurable velocity
(e.g., if a class has been mocked 5 times and not mocked 3         for data-driven project management.
times, it will be categorized as ‘mocked’). To compare the two        Kim et al. [26] stated that unit testing within the embedded
sets, we use the Wilcoxon rank sum test [42] (with confidence      systems industry poses several unique challenges: software is
level of 95%) and Cliff’s delta [22] to measure the effect size.   often developed on a different machine than it will run on
We choose Wilcoxon since it is a non-parametric test (does         and it is tightly coupled with the target hardware. This study
not have any assumption on the underlying data distribution).      shows how unit testing techniques and mocking frameworks
   As a result, we see that both mocked and non mocked             can facilitate the design process, increase code coverage and
classes are similar in terms of coupling: The mean coupling        the protection against regression defects.
of mocked classes is 5.89 with a maximum of 51, while the             Tim et al. [28] stated that using Mock Objects is the only
mean coupling of non mocked classes is even slightly higher        way to unit test domain code that depends on state that is
(7.131) with a maximum of 187. However, from the Wilcoxon          difficult or impossible to reproduce. They show that the usage
rank sum test and the effect size, we observe that the over-       of mocks encourages better-structured tests and reduces the
all difference is negligible (Wilcoxon p-value<0.001, Cliff’s      cost of writing stub code, with a common format for unit
Delta=−0.121). Same happens for the complexity metrics: The        tests that is easy to learn and understand.
mean complexity of mocked classes is 10.58 with a maximum
                                                                                        VII. C ONCLUSION
of 89.00, while the mean complexity of non mocked classes
is 16.42 (max 420). Difference is also negligible (Wilcoxon           Mocking is a common testing practice among software
p-value=5.945e−07 , Cliff’s delta=−0.166).                         developers. However, there is little empirical evidence on how
                                                                   developers actually apply the technique in their software sys-
   We conjecture that the chosen code metrics are not enough       tems. We investigated how and why developers currently use
to predict whether a class should be mocked. Future research       mock objects. To that end, we studied three OSS projects and
needs to be conducted to understand how code metrics are           one industrial system, interviewed three of their developers,
related to mocking decisions.                                      surveyed 105 professionals, and discussed the findings with a
  There are many other discoveries that can be verified using      main developer from the leading Java mocking framework.
quantitative studies (i.e. are slow tests mocked more often?          Our results show that developers tend to mock dependencies
how faster are test that use mocks?). Here we simply proposed      that make testing difficult, i.e., classes that are hard to set
an initial analysis to show its feasibility. Further research      up or that depend on external resources. In contrast, devel-
can be designed and carried out to devise approaches to            opers do not often mock classes that they can fully control.
quantitatively evaluate mocking practices.                         Interestingly, a class being slow is not an important factor
for developers when mocking. As for challenges, developers                     [22] M. R. Hess and J. D. Kromrey. Robust Confidence Intervals for Effect
affirm that challenges when mocking are mostly technical,                           Sizes: A Comparative Study of Cohen’s d and Cliff’s Delta Under
                                                                                    Non-normality and Heterogeneous Variances. American Educational
such as dealing with unstable dependencies, the coupling                            Research Association, San Diego, nov 2004.
between the mock and the production code, legacy systems,                      [23] A. Hunt and D. Thomas. Pragmatic unit testing in c# with nunit. The
and hard-to-test classes are the most important ones.                               Pragmatic Programmers, 2004.
                                                                               [24] T. Kaczanowski. Practical Unit Testing with TestNG and Mockito.
   Our future agenda includes understanding the relationship                        Tomasz Kaczanowski, 2012.
between code quality metrics and the use of mocking as                         [25] M. Karlesky, G. Williams, W. Bereza, and M. Fletcher. Mocking the
well as the role of software evolution in developers’ mocking                       Embedded World: Test-Driven Development, Continuous Integration,
                                                                                    and Design Patterns. In Embedded Systems Conference Silicon Valley
practices.                                                                          (San Jose, California) ESC 413, April 2007. ESC 413, 2007.
                                                                               [26] S. S. Kim. Mocking embedded hardware for software validation. PhD
                             R EFERENCES                                            thesis, 2016.
 [1] Alura. http://www.alura.com.br/. [Online; accessed 03-Feb-2016].          [27] J. Langr, A. Hunt, and D. Thomas. Pragmatic Unit Testing in Java 8
 [2] EasyMock. http://easymock.org. [Online; accessed 03-Feb-2016].                 with JUnit. Pragmatic Bookshelf, 2015.
 [3] JMock. http://www.jmock.org. [Online; accessed 03-Feb-2016].              [28] T. Mackinnon, S. Freeman, and P. Craig. Endo-testing: unit testing with
 [4] Kotlin. https://kotlinlang.org. [Online; accessed 03-Feb-2016].                mock objects. Extreme programming examined, pages 287–301, 2001.
 [5] Mock. https://github.com/testing-cabal/mock. [Online; accessed 03-Feb-    [29] M. R. Marri, T. Xie, N. Tillmann, J. De Halleux, and W. Schulte. An
     2016].                                                                         empirical study of testing file-system-dependent software with mock
 [6] Mocker. https://labix.org/mocker. [Online; accessed 03-Feb-2016].              objects. AST, 9:149–153, 2009.
 [7] Mockito. http://site.mockito.org. [Online; accessed 03-Feb-2016].         [31] G. Meszaros. xUnit test patterns: Refactoring test code. Pearson
 [8] MyBatis. http://www.mybatis.org/. [Online; accessed 03-Feb-2016].              Education, 2007.
 [9] PowerMock. https://github.com/powermock/powermock. [Online; ac-           [32] S. Mostafa and X. Wang. An Empirical Study on the Usage of Mocking
     cessed 03-Feb-2016].                                                           Frameworks in Software Testing. In 2014 14th International Conference
[10] Sonarqube. https://www.sonarqube.org/. [Online; accessed 03-Feb-               on Quality Software, pages 127–132. IEEE, oct 2014.
     2016].                                                                    [33] A. J. Nederhof. Methods of coping with social desirability bias: A
[11] Spring Framework. https://projects.spring.io/spring-framework/. [On-           review. European journal of social psychology, 15(3):263–280, 1985.
     line; accessed 03-Feb-2016].                                              [34] R. Osherove. The Art of Unit Testing: With Examples in .NET. Manning,
[12] To Mock or Not To Mock? Online Appendix. https://doi.org/10.4121/              2009.
     uuid:fce8653c-344c-4dcb-97ab-c9c1407ad2f0.                                [35] G. Rugg. A rticle picture sorts and item sorts. Computing, 22(3), 2005.
[13] VRaptor. https://www.vraptor.com.br/. [Online; accessed 03-Feb-2016].     [36] P. Runeson. A survey of unit testing practices. IEEE Software, 23(4):22–
[14] K. Beck. Test-driven development: by example. Addison-Wesley                   29, 2006.
     Professional, 2003.                                                       [37] H. Samimi, R. Hicks, A. Fogel, and T. Millstein. Declarative Mocking
[15] E. Evans. Domain-driven design: tackling complexity in the heart of            Categories and Subject Descriptors. pages 246–256, 2013.
     software. Addison-Wesley Professional, 2004.
                                                                               [38] D. Spadini, M. Aniche, A. Bacchelli, and M. Bruntink. MockExtractor.
[16] M. Fowler. Patterns of enterprise application architecture. Addison-
                                                                                    The tool is available at https://github.com/ishepard/MockExtractor.
     Wesley Longman Publishing Co., Inc., 2002.
                                                                               [39] D.      Spencer.            Card      sorting:   a   definitive   guide.
[17] S. Freeman, T. Mackinnon, N. Pryce, and J. Walnes. Mock roles, objects.
                                                                                    http://boxesandarrows.com/card-sorting-a-definitive-guide/, 2004.
     In Companion to the 19th annual ACM SIGPLAN conference on Object-
     oriented programming systems, languages, and applications, pages 236–     [40] K. Taneja, Y. Zhang, and T. Xie. MODA: Automated Test Generation
     246. ACM, 2004.                                                                for Database Applications via Mock Objects. In Proceedings of the
[18] S. Freeman and N. Pryce. Growing object-oriented software, guided by           IEEE/ACM international conference on Automated software engineering
     tests. Pearson Education, 2009.                                                - ASE ’10, page 289, New York, New York, USA, 2010. ACM Press.
[19] P. Hamill. Unit Test Frameworks: Tools for High-Quality Software          [41] E. Weyuker. Testing component-based software: a cautionary tale. IEEE
     Development. O’Reilly Media, 2004.                                             Software, 15(5):54–59, 1998.
[20] B. Hanington and B. Martin. Universal methods of design: 100 ways         [42] F. Wilcoxon. Individual comparisons of grouped data by ranking
     to research complex problems, develop innovative ideas, and design             methods. Journal of economic entomology, 39(6):269, 1946.
     effective solutions. Rockport Publishers, 2012.                           [43] A. Zaidman, B. Van Rompaey, S. Demeyer, and A. Van Deursen. Mining
[21] F. Henderson. Software Engineering at Google. feb 2017.                        software repositories to study co-evolution of production & test code.
[30] T. McCabe. A Complexity Measure. IEEE Transactions on Software                 In 2008 1st International Conference on Software Testing, Verification,
     Engineering, SE-2(4):308–320, dec 1976.                                        and Validation, pages 220–229. IEEE, 2008.
