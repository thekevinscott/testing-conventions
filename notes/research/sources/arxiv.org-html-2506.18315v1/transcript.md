---
url: https://arxiv.org/html/2506.18315v1
title: Use Property-Based Testing to Bridge LLM Code Generation and Validation
fetched: 2026-06-27
raw: raw.html
transport: curl
capture_status: ok
---

1. [I INTRODUCTION](https://arxiv.org/html/2506.18315v1#S1 "In Use Property-Based Testing to Bridge LLM Code Generation and Validation")
2. [II MOTIVATING EXAMPLE](https://arxiv.org/html/2506.18315v1#S2 "In Use Property-Based Testing to Bridge LLM Code Generation and Validation")
3. [III FRAMEWORK](https://arxiv.org/html/2506.18315v1#S3 "In Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   1. [III-A Preliminaries](https://arxiv.org/html/2506.18315v1#S3.SS1 "In III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   2. [III-B Framework Overview](https://arxiv.org/html/2506.18315v1#S3.SS2 "In III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   3. [III-C Tester: Property-Based Testing and Feedback Generation](https://arxiv.org/html/2506.18315v1#S3.SS3 "In III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   4. [III-D Generator: Code Generation and Refinement](https://arxiv.org/html/2506.18315v1#S3.SS4 "In III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
4. [IV EXPERIMENTS AND RESULTS](https://arxiv.org/html/2506.18315v1#S4 "In Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   1. [IV-A Experiment Settings](https://arxiv.org/html/2506.18315v1#S4.SS1 "In IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
      1. [IV-A1 Benchmarks](https://arxiv.org/html/2506.18315v1#S4.SS1.SSS1 "In IV-A Experiment Settings ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
      2. [IV-A2 Metrics](https://arxiv.org/html/2506.18315v1#S4.SS1.SSS2 "In IV-A Experiment Settings ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
      3. [IV-A3 Foundation Models](https://arxiv.org/html/2506.18315v1#S4.SS1.SSS3 "In IV-A Experiment Settings ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
      4. [IV-A4 Comparison Baselines](https://arxiv.org/html/2506.18315v1#S4.SS1.SSS4 "In IV-A Experiment Settings ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
      5. [IV-A5 PGS Implementation Details](https://arxiv.org/html/2506.18315v1#S4.SS1.SSS5 "In IV-A Experiment Settings ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   2. [IV-B RQ1: Overall Performance](https://arxiv.org/html/2506.18315v1#S4.SS2 "In IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   3. [IV-C RQ2: Effectiveness of Property-Driven Validation](https://arxiv.org/html/2506.18315v1#S4.SS3 "In IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   4. [IV-D RQ3: The Viability and Impact of LLM-Generated Property](https://arxiv.org/html/2506.18315v1#S4.SS4 "In IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   5. [IV-E RQ4: Performance Across Task Difficulty and LLMs](https://arxiv.org/html/2506.18315v1#S4.SS5 "In IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
   6. [IV-F Threats to Validity](https://arxiv.org/html/2506.18315v1#S4.SS6 "In IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")
5. [V RELATED WORK](https://arxiv.org/html/2506.18315v1#S5 "In Use Property-Based Testing to Bridge LLM Code Generation and Validation")
6. [VI CONCLUSION](https://arxiv.org/html/2506.18315v1#S6 "In Use Property-Based Testing to Bridge LLM Code Generation and Validation")

# Use Property-Based Testing to Bridge LLM Code Generation and Validation

Lehan He1
  
Jing Shao
School of Software, Beihang University
  
Shanghai Innovation Institute
  
Beijing, China
  
helehan@buaa.edu.cn
Shanghai AI Laboratory
  
Shanghai Innovation Institute
  
Shanghai, China
  
shaojing@pjlab.org.cn
  

Zeren Chen1
  
Xiang Gao2
School of Software, Beihang University
  
Shanghai AI Laboratory
  
Beijing, China
  
czr1604@buaa.edu.cn
School of Software, Beihang University
  
Beijing, China
  
xiang\_gao@buaa.edu.cn
  
Zhe Zhang
  
  
Lu Sheng2
1These authors contributed equally to this work.
2Corresponding author.
School of Software, Beihang University
  
Beijing, China
  
zhangzhe2023@buaa.edu.cn
School of Software, Beihang University
  
Beijing, China
  
lsheng@buaa.edu.cn

###### Abstract

Large Language Models (LLMs) excel at code generation, but ensuring their outputs to be functionally correct, especially in complex programming tasks, is a persistent challenge.
While traditional Test-Driven Development (TDD) offers a path for code refinement, its efficacy with LLMs is often undermined by the scarcity of high-quality test cases or the pitfalls of automated test generation, including biased tests or inaccurate output predictions that can misdirect the correction process.
This paper introduces Property-Generated Solver, a novel framework that leverages Property-Based Testing (PBT) to validate high-level program properties or invariants, instead of relying on specific input-output examples.
These properties are often simpler to define and verify than directly predicting exhaustive test oracles, breaking the “cycle of self-deception” where tests might share flaws with the code they are meant to validate.
Property-Generated Solver employs two collaborative LLM-based agents: a Generator dedicated to code generation and iterative refinement, and a Tester that manages the PBT life-cycle and formulate semantically rich feedback from property violations.
The resulting comprehensive and actionable feedback then guides the Generator in its refinement efforts.
By establishing PBT as the core validation engine within this iterative, closed-loop paradigm, Property-Generated Solver provides a robust mechanism for steering LLMs towards more correct and generalizable code.
Extensive experimental results on multiple code generation benchmarks demonstrate that Property-Generated Solver achieves substantial pass@1 improvements, ranging from 23.1% to 37.3% relative gains over established TDD methods.

###### Index Terms:

Code Generation, Large Language Models, Agent, Property-Based Testing, Software Engineering

## I INTRODUCTION

Recent advances in Large Language Models (LLMs) have revolutionized automated code generation, enabling tools like GitHub Copilot to assist developers in translating natural language requirements into functional code [[1](https://arxiv.org/html/2506.18315v1#bib.bib1), [2](https://arxiv.org/html/2506.18315v1#bib.bib2), [3](https://arxiv.org/html/2506.18315v1#bib.bib3)].
However, ensuring the correctness of the generated code remains a critical and pressing challenge [[4](https://arxiv.org/html/2506.18315v1#bib.bib4)].
Test-Driven Development (TDD) [[5](https://arxiv.org/html/2506.18315v1#bib.bib5), [6](https://arxiv.org/html/2506.18315v1#bib.bib6), [7](https://arxiv.org/html/2506.18315v1#bib.bib7)], which leverages test cases and corresponding execution results to iteratively refine auto-generated code, has shown promise in enhancing its correctness.
Yet, existing methods suffer from a significant flaw: high-quality test cases are not always available.
Incomplete or biased feedback from these test results can even misguide the refinement process, potentially trapping LLMs in local optima and consequently hindering their ability to produce robust solutions.

Recent approaches have attempted to alleviate this issue by automatically generating numerous test cases, derived either from natural language problem specifications [[8](https://arxiv.org/html/2506.18315v1#bib.bib8), [9](https://arxiv.org/html/2506.18315v1#bib.bib9), [10](https://arxiv.org/html/2506.18315v1#bib.bib10)] or, in some instances, from the potentially flawed code itself [[11](https://arxiv.org/html/2506.18315v1#bib.bib11), [12](https://arxiv.org/html/2506.18315v1#bib.bib12), [13](https://arxiv.org/html/2506.18315v1#bib.bib13)].
However, the automated test case generation still faces several key challenges.
(1) Test case generation process can inadvertently mirror the code generation process, especially if both rely on similar underlying models or logic.
This may lead to a “cycle of self-deception”, where test cases share the same biases or misunderstandings as the generated code, thus failing to expose its critical flaws.
(2) Accurately generating test oracle (*i.e.*, expected outputs) can be even more challenging than the initial code generation [[14](https://arxiv.org/html/2506.18315v1#bib.bib14), [15](https://arxiv.org/html/2506.18315v1#bib.bib15)].
Indeed, employing LLMs, even with advanced techniques like Chain-of-Thought (CoT) reasoning, for reliable oracle prediction has often proven to be unreliable or computationally infeasible [[16](https://arxiv.org/html/2506.18315v1#bib.bib16)].
(3) Existing test case generation techniques tend to prioritize maximizing structural code coverage over verifying semantic validity.
The resulting tests are often insufficient to validate functional correctness, especially when it comes to detecting subtle logical errors that LLMs may introduce.

Instead of validating specific input-output pairs in test cases, Property-Based Testing (PBT) [[17](https://arxiv.org/html/2506.18315v1#bib.bib17)] focuses on high-level properties or invariants that the code must satisfy for any valid input.
For instance, a fundamental property of a sorting function is that “sorting a list always returns a non-decreasing sequence.”
A PBT framework verifies the sorting function’s output is indeed non-decreasing, bypassing the need to predict the exact sorted output for every input [[18](https://arxiv.org/html/2506.18315v1#bib.bib18)].
Defining such property is typically less complex than predicting exhaustive test oracles, as these properties capture essential correctness characteristics without requiring precise input-output mappings.
For instance, creating exhaustive oracles for NP-hard problems (*e.g.*, optimal graph coloring) by predicting correct outputs for all inputs is often intractable.
However, one can easily define a verifiable property like “no two adjacent nodes in a colored graph share the same color.”
Such a property validates a crucial aspect of correctness and effectively constrains the solution space, without requiring the underlying hard problem to be solved for each test case.

Motivated by the advantages of PBT, we introduce Property-Generated Solver (PGS), a novel framework that embeds PBT as a core engine for an iterative, LLM-driven code generation and refinement process.
PGS employs two key agents—a Generator and a Tester—that systematically decouple code generation from its validation.
After the Generator produces initial candidate programs, these agents collaborate iteratively: the Tester rigorously validates them using defined properties, while the Generator refines the programs based on the feedback from validation results.
Specifically, the Tester manages the PBT life-cycle:
it defines high-level abstract properties (*e.g.*, invariants, functional constraints) that serve as precise specifications.
The Tester then translates them into corresponding executable property-checking code and generates diverse test inputs to instantiate these properties, against which the candidate programs are executed.
After strategically selecting property violations from execution results, the Tester provides semantically rich feedback and high-level insights, effectively guiding the Generator’s subsequent refinement.
The iterative cycle of property definition, instantiation, and code refinement continues until the program satisfies all properties or a predefined budget is exhausted.
By grounding generation and refinement in such a property-centric approach, PGS steers the LLM towards more robust and correct solutions.

Comprehensive experimental results demonstrate that proposed PGS framework significantly enhances the robustness and quality of generated code in real-world tasks.
We evaluate PGS across multiple code generation benchmarks (HumanEval [[19](https://arxiv.org/html/2506.18315v1#bib.bib19)], MBPP [[20](https://arxiv.org/html/2506.18315v1#bib.bib20)] and LivecodeBench [[16](https://arxiv.org/html/2506.18315v1#bib.bib16)]) of varying difficulty using LLMs with different capabilities, showcasing that PGS achieves a 23.1%–37.3% relative improvement (pass@1) over previous TDD methods on problems that are challenging for direct prompting approaches.

In summary, the key contributions in this paper are summarized as follows:

* •

  We propose PGS, a novel framework that, to the best of our knowledge, is the first to systematically apply Property-Based Testing as the primary driver for LLM-based code generation and refinement.
  PGS achieves this through two collaborative agents: a Generator dedicated to code generation and refinement, and a Tester responsible for property-driven validation.
* •

  We investigate and demonstrate how feedback derived from property-driven validation provides more effective guidance for LLM-based code generation compared to feedback from conventional TDD methods.
* •

  PGS achieves new state-of-the-art (SOTA) results on multiple code generation benchmarks of varying difficulty.
  To foster reproducibility and further research, the source code and data are available on github. repository111<https://github.com/HeLeHanPrivate/PBTwithCodeGen>.

![Refer to caption](x1.png)

Figure 1: 
A programming problem excerpted from the HumanEval [[19](https://arxiv.org/html/2506.18315v1#bib.bib19)] benchmark (#test25). This illustrates a scenario where defining properties can more robustly guide code correction than relying on limited example tests, motivating our approach.

## II MOTIVATING EXAMPLE

Consider the problem of integer prime factorization, *e.g.*, HumanEval/25, illustrated in [Figure 1](https://arxiv.org/html/2506.18315v1#S1.F1 "In I INTRODUCTION ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation").
This task requires finding all prime factors of a given integer, presented in ascending order, with each distinct prime factor appearing according to its multiplicity in the original number.
For instance, factorize(12) should yield [2, 2, 3] (2×2×3=12223122\times 2\times 3=122 × 2 × 3 = 12).

Existing TDD approaches [[21](https://arxiv.org/html/2506.18315v1#bib.bib21)] typically involve employing LLMs to generate specific input-output test cases (*e.g.*, assert factorize(12) == [2,2,3]), and using the test execution results to guide subsequent code refinement.
However, when applied to this factorization problem, particularly with automatically generated tests, this methodology encounters critical limitations.
First, automated test case generation can struggle to produce correct oracles.
If the LLM tasked with generating tests shares the same logical misunderstandings as its code-generation counterpart, it might produce an incorrect test oracle, such as assert factorize(12) == [2,3], which omits the requirements of repeating based on multiplicity.
This creates a “cycle of self-deception” where flawed code is validated against equally flawed tests, failing to expose the underlying error.
Second, even with a “Ground Truth” oracle, a simple fail signal offers limited semantic insight for code refinement.
The failed assertion primarily indicates a value mismatch but does not explicitly explain why it is wrong in terms of the problem’s semantics (*e.g.*, “multiplicity error”).
Especially for LLM, which may have limitations in complex mathematical reasoning [[22](https://arxiv.org/html/2506.18315v1#bib.bib22)], deducing the precise nature of the logical error from such feedback can be difficult.
This often results in inefficient trial-and-error refinement, ultimately leading to suboptimal code generation results.

In contrast, property-based testing often avoids this trap by validating invariant properties rather than specific examples.
For factorization problem, one critical property is product equivalence: “the product of the output factors must equal the original input integer.”
By grounding validation in these properties, we leverage PBT for a more robust code generation and refinement framework to address above challenges.

Our framework, PGS, is designed to harness these advantages of PBT to create a more reliable LLM-driven code generation and refinement process.
PGS achieves this by employing two distinct LLM-powered agents: a Generator responsible for code synthesis and modification, and a Tester dedicated to orchestrating the PBT-driven validation and feedback generation.
It ensures that the standard for correctness is independent of the code generation process’s potential biases and provides more insightful guidance for the LLM to overcome logical errors, leading to more robust and correct solutions.

![Refer to caption](x2.png)

Figure 2: 
Overview of the Property-Generated Solver framework, showcasing the iterative collaboration between the Generator and the Tester.

## III FRAMEWORK

### III-A Preliminaries

Problem Definition.
The primary objective in the code generation task is to employ LLMs to generate a program C𝐶Citalic\_C based on a given natural language specification Q𝑄Qitalic\_Q and a set of public (visible) test cases 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT, where each test case ti=(Ii,Oi)subscript𝑡𝑖subscript𝐼𝑖subscript𝑂𝑖t\_{i}=(I\_{i},O\_{i})italic\_t start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT = ( italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT , italic\_O start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT ) consists of an input Iisubscript𝐼𝑖I\_{i}italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT and an expected output Oisubscript𝑂𝑖O\_{i}italic\_O start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT.
The generated program C𝐶Citalic\_C is then evaluated against a set of private (hidden) test cases 𝑻hsubscript𝑻ℎ\bm{T}\_{h}bold\_italic\_T start\_POSTSUBSCRIPT italic\_h end\_POSTSUBSCRIPT, and is judged correct if it passes all tj∈𝑻hsubscript𝑡𝑗subscript𝑻ℎt\_{j}\in\bm{T}\_{h}italic\_t start\_POSTSUBSCRIPT italic\_j end\_POSTSUBSCRIPT ∈ bold\_italic\_T start\_POSTSUBSCRIPT italic\_h end\_POSTSUBSCRIPT (*i.e.*, satisfying ∀tj=(Ij,Oj)∈𝑻h,C⁢(Ij)=Ojformulae-sequencefor-allsubscript𝑡𝑗subscript𝐼𝑗subscript𝑂𝑗subscript𝑻ℎ𝐶subscript𝐼𝑗subscript𝑂𝑗\forall t\_{j}=(I\_{j},O\_{j})\in\bm{T}\_{h},C(I\_{j})=O\_{j}∀ italic\_t start\_POSTSUBSCRIPT italic\_j end\_POSTSUBSCRIPT = ( italic\_I start\_POSTSUBSCRIPT italic\_j end\_POSTSUBSCRIPT , italic\_O start\_POSTSUBSCRIPT italic\_j end\_POSTSUBSCRIPT ) ∈ bold\_italic\_T start\_POSTSUBSCRIPT italic\_h end\_POSTSUBSCRIPT , italic\_C ( italic\_I start\_POSTSUBSCRIPT italic\_j end\_POSTSUBSCRIPT ) = italic\_O start\_POSTSUBSCRIPT italic\_j end\_POSTSUBSCRIPT).
Beyond initial generation, LLMs can perform ranking, filtering, or refining program C𝐶Citalic\_C.
These actions can be based on execution feedback from 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT or on test cases the LLM itself generates.

LLM Agents.
LLM-based agents are autonomous systems that integrate the reasoning capabilities of LLMs with specialized external tools [[23](https://arxiv.org/html/2506.18315v1#bib.bib23)].
These agents typically utilize predefined prompts to enable interaction with users or other systems.
In the PGS framework, agents leverage available context, such as the problem specification and execution feedback, to guide the LLMs through iterative cycles of property-driven validation, feedback formulation and code refinement.

### III-B Framework Overview

As illustrated in [Figure 2](https://arxiv.org/html/2506.18315v1#S2.F2 "In II MOTIVATING EXAMPLE ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation"), PGS comprises two primary LLM-powered agents: a Generator and a Tester.
Both agents can be implemented using general-purpose LLMs such as GPT-4 [[1](https://arxiv.org/html/2506.18315v1#bib.bib1)] or DeepSeek-R1 [[24](https://arxiv.org/html/2506.18315v1#bib.bib24)].
The core idea is to leverage Property-Based Testing to overcome the limitations of traditional test case generation, particularly in predicting accurate oracles and ensuring semantic validation.
PGS achieves this through a clear separation of concerns: the Generator handles code generation and refinement, while the Tester manages property definition and feedback formulation.

The process begins with the Generator generating an initial candidate program C𝐶Citalic\_C based on the problem description Q𝑄Qitalic\_Q.
Following this, an iterative workflow unfolds:

1. 1.

   Property Definition:
   Concurrently with the initial code generation, the Tester defines high-level, abstract properties 𝒫𝒫\mathcal{P}caligraphic\_P derived from Q𝑄Qitalic\_Q.
2. 2.

   Property Instantiation:
   The Tester translates the defined properties 𝒫𝒫\mathcal{P}caligraphic\_P into corresponding executable property-checking code C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT and then dynamically synthesizes a diverse set of PBT inputs {IiPBT}subscriptsuperscript𝐼PBT𝑖\{I^{\text{PBT{}}}\_{i}\}{ italic\_I start\_POSTSUPERSCRIPT PBT end\_POSTSUPERSCRIPT start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT } that adhere to problem constraints.
3. 3.

   Property-driven Validation:
   The Generator then validates C𝐶Citalic\_C against the defined properties 𝒫𝒫\mathcal{P}caligraphic\_P using the property-checking code C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT, identifying any violations triggered by synthesized PBT inputs and public tests 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT.
4. 4.

   Feedback Formulation:
   The Tester analyzes all execution results and then strategically selects the most informative failing cases formulate comprehensive and actionable feedback for Generator.
5. 5.

   Code Refinement:
   Generator attempts to refine its program C𝐶Citalic\_C based on the feedback received from the Tester.

This iterative cycle continues until the program C𝐶Citalic\_C satisfies all checks (𝒫𝒫\mathcal{P}caligraphic\_P and 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT) or a predefined budget is exhausted.

### III-C Tester: Property-Based Testing and Feedback Generation

The Tester is pivotal in orchestrating the testing strategy within PGS.
Its responsibilities include defining verifiable properties, translating them into executable property-checking code, generating diverse PBT inputs, and formulating actionable feedback to guide the Generator.

Property Definition.
Given the natural language specification Q𝑄Qitalic\_Q, the Tester primarily employs LLM-Assisted Property Generation.
Specifically, it prompts the LLM with Q𝑄Qitalic\_Q to generate candidate properties 𝒫𝒫\mathcal{P}caligraphic\_P.
These properties can range from invariants covering the entire specification to those addressing partial aspects.
The prompt used for this definition is detailed in [Figure 3](https://arxiv.org/html/2506.18315v1#S3.F3 "In III-C Tester: Property-Based Testing and Feedback Generation ‣ III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation").
While our framework can accommodate human-provided properties, it emphasizes automating their generation via LLMs to enhance scalability and reduce manual effort.

Property Instantiation.
Following the definition of abstract properties 𝒫𝒫\mathcal{P}caligraphic\_P, the Tester translates them into executable property-checking code, denoted as C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT.
This transformation typically involves structuring the properties as assertion statements, boolean-valued verification functions, or other forms of logical checks that can be programmatically evaluated.
Before this executable property-checking code C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT is used for validating Generator’s generated code C𝐶Citalic\_C, a crucial validation step is performed by the Tester.
This step aims to filter out property-checking code that might contradict known ground truth from public test cases 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT (if available) or lack sensitivity to actual errors.
Specifically, the Tester assesses each C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT based on the criteria:

1. 1.

   Soundness against Public Tests:
   For every correct input-output pair (Ii,Oi)∈𝑻vsubscript𝐼𝑖subscript𝑂𝑖subscript𝑻𝑣(I\_{i},O\_{i})\in\bm{T}\_{v}( italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT , italic\_O start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT ) ∈ bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT, C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT (when applied to Oisubscript𝑂𝑖O\_{i}italic\_O start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT given Iisubscript𝐼𝑖I\_{i}italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT) must evaluate to True.
   This ensures C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT does not falsely reject known good behaviors.
2. 2.

   Sensitivity to Known Errors:
   For an erroneous output Oierrsuperscriptsubscript𝑂𝑖errO\_{i}^{\text{err}}italic\_O start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT start\_POSTSUPERSCRIPT err end\_POSTSUPERSCRIPT produced by a flawed version of C𝐶Citalic\_C from a previous iteration, C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT (when applied to Oierrsuperscriptsubscript𝑂𝑖errO\_{i}^{\text{err}}italic\_O start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT start\_POSTSUPERSCRIPT err end\_POSTSUPERSCRIPT given Iisubscript𝐼𝑖I\_{i}italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT), should ideally evaluate to False.
   This ensures C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT is capable of detecting known types of errors.

Once the set of property-checking code C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT is validated, the Tester proceeds to PBT input synthesis.
It generates a diverse set of inputs {IiPBT}subscriptsuperscript𝐼PBT𝑖\{I^{\text{PBT{}}}\_{i}\}{ italic\_I start\_POSTSUPERSCRIPT PBT end\_POSTSUPERSCRIPT start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT } designed to effectively exercise the properties.
This is often achieved by prompting the LLM to create a test input generator script, inspired by approaches like [[25](https://arxiv.org/html/2506.18315v1#bib.bib25)].
An example prompt for generating such scripts is shown in [Figure 3](https://arxiv.org/html/2506.18315v1#S3.F3 "In III-C Tester: Property-Based Testing and Feedback Generation ‣ III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation").
The Tester then provides the validated property-checking code C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT and the synthesized inputs {IiPBT}subscriptsuperscript𝐼PBT𝑖\{I^{\text{PBT{}}}\_{i}\}{ italic\_I start\_POSTSUPERSCRIPT PBT end\_POSTSUPERSCRIPT start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT } to the Generator, which will use them in the property-driven validation step

![Refer to caption](x3.png)

Figure 3: 
The prompt template used by the Tester to generate validation and input generator.

Feedback Formulation.
Following the property-driven validation performed by the Generator, the Tester gathers all execution results from Generator.
The Tester analyzes these results, particularly any identified property violations or failures of public test cases 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT, to formulate comprehensive and actionable feedback to guide the Generator’s subsequent code refinement.
These feedback, constructed from a selected failing case from validation results, typically includes:
(1) An input Iisubscript𝐼𝑖I\_{i}italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT that led to a property violation or test failure.
(2) The observed erroneous output Oierr=C⁢(Ii)subscriptsuperscript𝑂err𝑖𝐶subscript𝐼𝑖O^{\text{err}}\_{i}=C(I\_{i})italic\_O start\_POSTSUPERSCRIPT err end\_POSTSUPERSCRIPT start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT = italic\_C ( italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT ).
(3) A description of the specific property Pjsubscript𝑃𝑗P\_{j}italic\_P start\_POSTSUBSCRIPT italic\_j end\_POSTSUBSCRIPT that was violated (and its corresponding check in C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT) or the public test case that failed.
Given that these raw validation results can be extensive or contain redundancies, strategically selecting which failing case to use for constructing feedback is crucial for effectively guiding the Generator’s refinement attempts.

Our investigation into feedback formulation strategies includes approaches common in traditional software testing, such as prioritizing failing cases with inputs that maximize code coverage.
However, we observe that they are often suboptimal for LLM-based refinement within our PBT-driven framework.
When presented with failing test cases, LLMs may exhibit behavior mimicking that of human programmers debugging, *i.e.*, tracing the execution path for the given input and attempting to identify where the generated logic diverged from the expected behavior.
However, overly complex and long execution paths, which can result from inputs designed to maximize coverage, could overwhelm the LLM, potentially leading it to get “lost in the middle” [[26](https://arxiv.org/html/2506.18315v1#bib.bib26)] of convoluted logic rather than pinpointing the core deficiencies.

Inspired by delta debugging principles [[27](https://arxiv.org/html/2506.18315v1#bib.bib27)], which aim to find the simplest input that still triggers a failure, our Tester therefore adopts a strategy of selecting failing cases with minimized error-inducing inputs.
Similar to how human programmer can benefit from such principle, our experiments (detailed in Section [IV](https://arxiv.org/html/2506.18315v1#S4 "IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation"), RQ2) reveal that LLMs can also tend to perform better with such simple yet straightforward feedback, since it provides a more direct and unambiguous signal of the fault while reducing extraneous information that can hinder the refinement process.

![Refer to caption](x4.png)

Figure 4: 
The prompt template used by the Generator to generate initial code and refine buggy code.

### III-D Generator: Code Generation and Refinement

The Generator agent is responsible for initially generating the program code based on the problem specification and, subsequently, for attempting to refine this code using the feedback provided by the Tester.

Initial Code Generation.
Given the natural language specification Q𝑄Qitalic\_Q, the Generator prompts an LLM to generate an initial candidate program C𝐶Citalic\_C.
The prompt used for this step is shown in [Figure 4](https://arxiv.org/html/2506.18315v1#S3.F4 "In III-C Tester: Property-Based Testing and Feedback Generation ‣ III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation").

Property-driven Validation.
To assess its current candidate program C𝐶Citalic\_C against the defined properties, the Generator first integrates the property-checking code C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT with C𝐶Citalic\_C.
Specifically, the Generator instructs an LLM to produce an instrumented version of the program, denoted C′superscript𝐶′C^{\prime}italic\_C start\_POSTSUPERSCRIPT ′ end\_POSTSUPERSCRIPT, allowing property violations can manifest as direct runtime errors (*e.g.*, AssertionError) during the execution of C′superscript𝐶′C^{\prime}italic\_C start\_POSTSUPERSCRIPT ′ end\_POSTSUPERSCRIPT.
The Generator then executes C′superscript𝐶′C^{\prime}italic\_C start\_POSTSUPERSCRIPT ′ end\_POSTSUPERSCRIPT against all relevant inputs: those from the public test cases 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT and the synthesized PBT inputs {IiPBT}subscriptsuperscript𝐼PBT𝑖\{I^{\text{PBT{}}}\_{i}\}{ italic\_I start\_POSTSUPERSCRIPT PBT end\_POSTSUPERSCRIPT start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT }.
Following execution, the Generator categorizes the overall execution results based on the observed behaviors:

* •

  Pass:
  C′superscript𝐶′C^{\prime}italic\_C start\_POSTSUPERSCRIPT ′ end\_POSTSUPERSCRIPT successfully passes all public test cases in 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT and does not trigger any violations of the integrated property checks C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT.
* •

  Property Violation:
  C′superscript𝐶′C^{\prime}italic\_C start\_POSTSUPERSCRIPT ′ end\_POSTSUPERSCRIPT fails an integrated property check from C𝒫subscript𝐶𝒫C\_{\mathcal{P}}italic\_C start\_POSTSUBSCRIPT caligraphic\_P end\_POSTSUBSCRIPT (*e.g.*, an AssertionError).
* •

  Wrong Answer on Tvsubscript𝑇𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT:
  For one or more test cases (Ii,Oi)∈𝑻vsubscript𝐼𝑖subscript𝑂𝑖subscript𝑻𝑣(I\_{i},O\_{i})\in\bm{T}\_{v}( italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT , italic\_O start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT ) ∈ bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT, the output C′⁢(Ii)≠Oisuperscript𝐶′subscript𝐼𝑖subscript𝑂𝑖C^{\prime}(I\_{i})\neq O\_{i}italic\_C start\_POSTSUPERSCRIPT ′ end\_POSTSUPERSCRIPT ( italic\_I start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT ) ≠ italic\_O start\_POSTSUBSCRIPT italic\_i end\_POSTSUBSCRIPT.
* •

  Runtime Error (Property-Irrelevant):
  The execution of C′superscript𝐶′C^{\prime}italic\_C start\_POSTSUPERSCRIPT ′ end\_POSTSUPERSCRIPT terminates prematurely due to errors unrelated to the integrated property checks (*e.g.*, IndexError or TypeError).
* •

  Time Limit Exceeded:
  C′superscript𝐶′C^{\prime}italic\_C start\_POSTSUPERSCRIPT ′ end\_POSTSUPERSCRIPT fails to produce an output within a predefined time limit.

Code Refinement.
Based on the feedback from Tester, the Generator prompts the LLM (as shown in [Figure 4](https://arxiv.org/html/2506.18315v1#S3.F4 "In III-C Tester: Property-Based Testing and Feedback Generation ‣ III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")) to generate a revised program.
This revised program becomes the new candidate C𝐶Citalic\_C for the subsequent iteration.
The LLM’s task is to address any identified property violations, incorrect answers on public tests, or runtime errors, while preserving functionality that adheres to the original specification Q𝑄Qitalic\_Q.
This collaborative iteration persists until the current version of the code achieves “Pass” after execution (successfully passing all public test cases 𝑻vsubscript𝑻𝑣\bm{T}\_{v}bold\_italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT and satisfies all defined properties 𝒫𝒫\mathcal{P}caligraphic\_P), or until a predefined stopping criterion, such as a maximum number of iterations or an overall time budget, is met.
The final version of the code generated before termination is then provided as the output.

## IV EXPERIMENTS AND RESULTS

To investigate how PGS leverages PBT to address fundamental challenges in robust LLM-based code generation, particularly achieving reliable PBT-driven validation and deep semantic correctness, we conduct comprehensive experiments exploring the following research questions (RQs):

* •

  RQ1: How does PGS perform against existing TDD methods in terms of generating correct code?
  This question addresses the primary claim of our work:
  integrating Property-Based Testing via PGS’s collaborative Generator and Tester can significantly enhance correctness in LLM-based code generation.
  We compare PGS’s effectiveness (pass@1, Repair Success Rate) on multiple code generation benchmarks against several Test-Driven Development baselines.
* •

  RQ2: How effective is the Property-Driven Validation?
  And which feedback formulation strategies are most effective for code refinement?
  We compare the guidance provided by PGS’s PBT-based feedback against those derived from public test cases and even inaccessible private test cases.
  Moreover, we conduct experiments to explore various feedback formulation strategies for selecting the most informative failing cases to guide subsequent refinement.
* •

  RQ3: How effectively can LLMs generate the property and corresponding checking code required by PGS?
  And what is their impact on code generation outcomes?
  Effectively leveraging PBT in PGS relies on the LLM’s capability to generate necessary PBT artifacts (*e.g.*, properties, checking code, input generators).
  First, we assess the LLM’s proficiency in generating valid and useful properties from problem specifications, particularly in comparison to its ability to directly generate the solution code.
  Second, we analyze how the quality and integration of these LLM-generated property checks within PGS influence the final code’s correctness and the distribution of different solution outcomes (*e.g.*, successful passes, property violations, or other failure modes).
* •

  RQ4: What is the generalizability of the proposed PGS framework across different LLMs and programming tasks of varying difficulty?
  We assess whether the performance benefits of PGS hold consistently when employing different LLMs for the Generator and Tester.
  Additionally, we evaluate PGS on multiple code generation benchmarks that span a range of difficulties, from simpler to more challenging tasks.

We first detail the experimental setup, including the benchmarks, baseline methods, and evaluation metrics.
We then address each research question, discussing corresponding results.

### IV-A Experiment Settings

To evaluate the effectiveness of PGS, we conducted comprehensive experiments on diverse code generation benchmarks using several LLMs with varying capabilities.
We compared PGS against several state-of-the-art baselines.

#### IV-A1 Benchmarks

Following prior works [[28](https://arxiv.org/html/2506.18315v1#bib.bib28)], our evaluation utilizes three prominent code generation benchmarks:

* •

  HumanEval [[19](https://arxiv.org/html/2506.18315v1#bib.bib19)]:
  A standard benchmark comprising 164 handwritten Python programming problems designed to evaluate the function-level code synthesis capabilities of LLMs.
  During the generation and refinement process, models are provided with the problem description [[29](https://arxiv.org/html/2506.18315v1#bib.bib29)] and any canonical tests accompanying the original HumanEval problem statements.
  Final validation is performed using the benchmark’s standard hidden test cases.
* •

  MBPP [[20](https://arxiv.org/html/2506.18315v1#bib.bib20)]:
  This benchmark consists of approximately 500 crowd-sourced entry-level Python programming problems.
  The models receive the problem description and the first hidden test case during the generation phase [[30](https://arxiv.org/html/2506.18315v1#bib.bib30)].
  Final validation is performed using the benchmark’s standard hidden test cases.
* •

  LiveCodeBench [[16](https://arxiv.org/html/2506.18315v1#bib.bib16)]:
  A challenging benchmark featuring problems sourced from live programming contests, often requiring more complex algorithmic reasoning, intricate I/O handling, and adherence to stricter execution constraints.
  To ensure a comprehensive and up-to-date evaluation, we utilize the latest “v5” version, comprising 880 problems.
  For all problems from this benchmark, the public test cases provided with each problem description are made available to all Test-Driven Development methods, including PGS and relevant baselines.

#### IV-A2 Metrics

We adopt two metrics to evaluate the effectiveness of PGS:

* •

  pass@1 [[31](https://arxiv.org/html/2506.18315v1#bib.bib31)] measures the overall proportion of problems for which the generated final code successfully passes all hidden (private) test cases.
* •

  Repair Success Rate (RSR) [[32](https://arxiv.org/html/2506.18315v1#bib.bib32)] quantifies the proportion of initially incorrect code samples that are successfully corrected by the iterative refinement process to pass all hidden test cases.

#### IV-A3 Foundation Models

We select three LLMs with different capabilities to implement proposed PGS.
Based on their general coding proficiency, they are listed from weak to strong as follows:

* •

  DeepSeek-Coder-V2 [[3](https://arxiv.org/html/2506.18315v1#bib.bib3)]:
  A powerful open-source model specifically optimized for code generation tasks.
* •

  Qwen2.5-Coder [[33](https://arxiv.org/html/2506.18315v1#bib.bib33)]:
  A strong open-source model from the Qwen series, known for its advanced coding abilities.
* •

  DeepSeek-R1-Distilled-32B [[24](https://arxiv.org/html/2506.18315v1#bib.bib24)]:
  A highly capable LLM featured with long CoT reasoning.
  We utilize a variant 32B distilled model, which aims to offer a strong balance of performance and efficiency.

For all models, we follow official configurations (*e.g.*, maximum context window of tokens, temperature, specific version identifiers) to guarantee a consistent setup.

#### IV-A4 Comparison Baselines

We compare PGS against the following baselines, which include direct prompting and several counterparts based on Test-Driven Development or debugging techniques:

* •

  Model Itself (Direct and CoT Prompting):
  It suggests the fundamental code generation capabilities of the LLM itself.
  We evaluate two primary zero-shot prompting approaches:
  (1) Direct Prompting:
  The LLM generates code directly from the problem description without explicit intermediate reasoning steps, serving as a fundamental baseline.
  (2) Chain-of-Thought Reasoning:
  We also employ CoT [[34](https://arxiv.org/html/2506.18315v1#bib.bib34)] prompting, which elicits LLMs to generate a chain of intermediate reasoning steps before producing the final code.
* •

  Code-T [[11](https://arxiv.org/html/2506.18315v1#bib.bib11)]:
  An approach that enhances code generation by leveraging automatically generated tests to guide the process.
* •

  Self-Edit [[35](https://arxiv.org/html/2506.18315v1#bib.bib35)]:
  A technique where the LLM attempts to refine its own generated code, typically based on execution feedback or self-critique.
* •

  Reflexion [[7](https://arxiv.org/html/2506.18315v1#bib.bib7)]:
  An CoT prompting approach that uses self-reflection on verbalized reasoning and test outcomes to iteratively improve code.
* •

  MGDebugger [[36](https://arxiv.org/html/2506.18315v1#bib.bib36)]:
  A multi-level debugging framework designed to enhance code correctness by identifying and fixing errors at different levels of code abstraction.
* •

  Self-Debugger [[37](https://arxiv.org/html/2506.18315v1#bib.bib37)]:
  An iterative method where LLMs are prompted to explain their code and fix bugs by simulating a rubber duck debugging process.
* •

  LDB [[6](https://arxiv.org/html/2506.18315v1#bib.bib6)]:
  A refinement technique that segments programs into basic blocks and tracks intermediate variable values during runtime to identify and repair errors.

All baselines are reproduced based on their publicly available implementations, where possible.
Besides, we provide identical problem descriptions and public test cases in respective benchmarks for all methods, for a fair comparison.

TABLE I: 
Overall performance comparison of PGS against baselines on HumanEval and MBPP across different LLMs.

|  |  |  |  |  |  |  |  |  |  |  |  |  |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
|  | DeepSeek-Coder-V2 | | | | Qwen2.5-Coder | | | | DeepSeek-R1-Distilled-32B | | | |
| Method | HumanEval | | MBPP | | HumanEval | | MBPP | | HumanEval | | MBPP | |
| pass@1 | RSR | pass@1 | RSR | pass@1 | RSR | pass@1 | RSR | pass@1 | RSR | pass@1 | RSR |
| Direct prompting | 76.2 | - | 56.8 | - | 87.8 | - | 59.4 | - | 93.3 | - | 63.8 | - |
| CoT prompting | 76.8 | - | 57.2 | - | 87.8 | - | 59.6 | - | 93.3 | - | 63.8 | - |
| Code-T | 81.1 | 20.6 | 60.4 | 8.4 | 88.4 | 5.0 | 61.0 | 3.9 | 94.5 | 18.2 | 68.8 | 13.8 |
| LDB | 82.3 | 25.6 | - | - | - | - | - | - | - | - | - | - |
| Self-Edit | 81.7 | 23.1 | 62.4 | 13.0 | 90.2 | 20.0 | 63.2 | 9.4 | 95.1 | 27.3 | 73.0 | 25.4 |
| MGDebugger | 83.5 | 39.9 | 63.8 | 16.2 | 92.1 | 35.0 | 64.2 | 11.8 | 95.7 | 36.4 | 73.6 | 27.1 |
| Self-Debugging | 84.1 | 33.2 | 63.8 | 16.2 | 92.7 | 40.0 | 64.4 | 12.3 | 96.3 | 45.5 | 74.4 | 29.3 |
| Reflexion | 86.6 | 43.7 | - | - | 91.5 | 30.0 | - | - | 95.1 | 27.3 | - | - |
| PGS(Ours) | 89.0 | 53.8 | 67.6 | 25.0 | 94.5 | 55.0 | 69.6 | 25.1 | 97.6 | 63.6 | 81.2 | 48.1 |

#### IV-A5 PGS Implementation Details

For the PGS framework, both the Generator and Tester roles are implemented using the LLMs mentioned above.
All generation tasks within PGS, from code generation and refinement by Generator to property instantiation by Tester, are conducted with a consistent temperature of 0.5 and a maximum generation limit of 32,768 tokens per LLM call.
The iterative refinement cycle between Generator and Tester is capped at a maximum of 5 iterations per problem.
During code execution, a 6-second time limit per test case is enforced.
Any executions exceeding this time limit result in a “Time Limit Exceeded” status.
For each problem, the Tester aims to generate up to 5 distinct property based on the problem description and subsequently synthesized 20 additional PBT inputs using LLM-generated script to challenge the current code against these properties.
Feedback from Tester to Generator prioritized the shortest input while still triggering a property violation.

### IV-B RQ1: Overall Performance

To answer RQ1, we evaluate PGS against various baselines on multiple benchmarks using pass@1 and Repair Success Rate (RSR).
The detailed results, presented in [Table I](https://arxiv.org/html/2506.18315v1#S4.T1 "In IV-A4 Comparison Baselines ‣ IV-A Experiment Settings ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation") and [Table II](https://arxiv.org/html/2506.18315v1#S4.T2 "In IV-B RQ1: Overall Performance ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation"), show that PGS consistently and significantly outperforms existing approaches across all tested LLMs and benchmarks.
On average, PGS achieves a substantial 9.2% absolute improvement in pass@1 scores over methods using prompting techniques.
For instance, gains range from 4.2% with Qwen2.5-Coder on LiveCodeBench to a notable 17.4% with DeepSeek-R1-Distilled-32B on MBPP.
Furthermore, PGS also demonstrates an average absolute RSR improvement of approximately 15.7% over representative TDD baselines on HumanEval and MBPP.
This highlights its superior ability to correct initially flawed code.
The consistent and significant advantage of PGS underscores the efficacy of its novel PBT-driven validation and feedback strategy facilitated by the collaborative Generator and Tester.

Comparison with Prompting Techniques.
Direct and CoT prompting approaches establish the baseline performance, reflecting the raw code generation capabilities of the LLMs.
PGS, along with other iterative refinement techniques, consistently surpasses these baselines.
This underscores a fundamental principle: iterative refinement guided by feedback is crucial for enhancing code correctness beyond initial generation, a principle that PGS leverages to its advantage through its specialized feedback mechanism.

Comparison with Existing TDD Methods.
PGS offers distinct advantages over existing TDD approaches through its superior mechanism for sourcing and utilizing feedback, leading to more effective code refinement.
Unlike techniques such as Code-T [[11](https://arxiv.org/html/2506.18315v1#bib.bib11)] that primarily use LLM-generated tests to rank multiple, independently generated code candidates, PGS focuses on iteratively refining a single solution.
Code-T’s ranking can be less effective if most initial candidates are flawed, making it difficult to identify or converge upon a truly correct output.
In contrast, PGS derives feedback from properties grounded in the problem specification itself, rather than relying on potentially erroneous outputs from other candidates.
This ensures a more objective and reliable validation standard for guiding the Generator.
Other TDD methods like Self-Edit [[35](https://arxiv.org/html/2506.18315v1#bib.bib35)] and Self-Debugging [[37](https://arxiv.org/html/2506.18315v1#bib.bib37)] typically validate code and guide refinement using feedback from a limited set of public test cases.
This can be insufficient for uncovering a diverse range of bugs or deep semantic errors.
PGS, however, leverages PBT to systematically generate a diverse and extensive set of test inputs based on specification-derived properties.
This approach yields feedback that is not only more comprehensive but also more abstract and semantically insightful.
By focusing on whether the code adheres to behavioral properties, PGS guides the Generator towards solutions that are logically sound and robust, leading to its superior RSR and overall pass@1 rates.

TABLE II: 
Performance comparison (pass@1) of PGS against baselines on LiveCodeBench across different LLMs and task difficulties.

| Method | DeepSeek-Coder-V2 | | | | Qwen2.5-Coder | | | | DeepSeek-R1-Distilled-32B | | | |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Easy | Medium | Hard | All | Easy | Medium | Hard | All | Easy | Medium | Hard | All |
| Direct prompting | 62.4 | 17.5 | 1.1 | 26.7 | 68.8 | 24.8 | 2.2 | 31.8 | 96.8 | 66.8 | 28.1 | 64.4 |
| Code-T | 67.7 | 19.3 | 1.5 | 29.2 | 70.3 | 25.4 | 2.6 | 32.6 | 98.2 | 72.5 | 30.7 | 67.8 |
| Self-Edit | 68.1 | 19.6 | 1.9 | 30.2 | 71.3 | 25.7 | 3.0 | 33.2 | 98.6 | 77.3 | 33.0 | 70.6 |
| Self-Debugging | 71.0 | 21.5 | 2.2 | 31.3 | 74.9 | 26.3 | 3.3 | 34.7 | 98.9 | 80.1 | 35.9 | 72.5 |
| PGS(Ours) | 73.5 | 23.0 | 2.6 | 32.7 | 77.1 | 27.8 | 3.7 | 36.0 | 99.3 | 81.3 | 40.7 | 74.5 |

### IV-C RQ2: Effectiveness of Property-Driven Validation

TABLE III: 
Effect of Input Selection Strategy

| Input Feature | Min | | Median | | Max | |
| --- | --- | --- | --- | --- | --- | --- |
| Pass | Token | Pass | Token | Pass | Token |
| Line Coverage | 72.7% | 3.45k | 72.2% | 3.56k | 72.1% | 3.64k |
| Runtime | 73.3% | 3.28k | 72.4% | 3.54k | 71.8% | 3.61k |
| Length | 74.5% | 3.24k | 72.1% | 3.57k | 71.5% | 3.66k |

![Refer to caption](x5.png)

Figure 5: 
Contribution of different testing and refinement stages to overall problem resolution on the LiveCodeBench (DeepSeek-R1-Distilled-32B) and HumanEval (Deepseek-Coder-V2). The PBT segment highlights its incremental contribution to achieving comprehensive correctness.

Impact of Property-Derived Tests on Refinement.
To assess the unique contribution of property-derived tests within our feedback-driven framework, we compare the refinement success achieved using PBT-generated feedback against that achieved using only the standard public test cases provided with the benchmarks.

Our findings highlight the significant impact of property-derived validation across both LiveCodeBench and HumanEval.
For this analysis, we focus on a challenging subset of problems: those that the LLM, without any feedback, fails to solve but could be resolved if theoretically perfect feedback from all hidden private test cases are available (*i.e.*, the sum of the ”+Public Tests”, ”+PBT”, and ”+Private Tests” in [Figure 5](https://arxiv.org/html/2506.18315v1#S4.F5 "In IV-C RQ2: Effectiveness of Property-Driven Validation ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")).
On LiveCodeBench, using feedback solely from public test cases allows for the correction of 46.6% of initially flawed instances.
Remarkably, when applying PBT-driven feedback to this same set of problems, the RSR within this subset further boosts to 75.9%.
These substantial improvements (also mirrors on HumanEval) underscores the power of PBT in creating effective additional validation.

By formulating feedback based on these properties, PGS significantly bridge the gap towards higher correctness where public tests alone fall short.

Exploring Optimal Feedback Formulation Strategies.
As mentioned in [Section III-C](https://arxiv.org/html/2506.18315v1#S3.SS3 "III-C Tester: Property-Based Testing and Feedback Generation ‣ III FRAMEWORK ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation"), extensive validation results after execution could contain numerous redundancies, compromising the refinement results.
Therefore, we investigate the optimal feedback formulation strategies, focusing on which type of failing test cases can offer the most guidance [[36](https://arxiv.org/html/2506.18315v1#bib.bib36)].
We compare several formulation strategies, including those prioritizing coverage, runtime, and input length.

The results, detailed in [Table III](https://arxiv.org/html/2506.18315v1#S4.T3 "In IV-C RQ2: Effectiveness of Property-Driven Validation ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation") (LiveCodeBench, DeepSeek-R1-Distilled-32B), reveal a consistent trend:
feedback derived from inputs with minimized characteristics typically leads to better refinement outcomes.
Specifically, selecting the failing input with the shortest length (“Min Length”) yields the best pass@1 (74.5%), an improvement of +2.4% over median length and +3.0% over the longest inputs.
This preference for brevity is mirrored by the minimum execution runtime strategy (“Min Runtime”), which also performs robustly (73.3% pass@1) and outperforms median/maximum runtime strategies.
Besides, both “Min Length” (3.24k tokens) and “Min Runtime” (3.28k tokens) strategies also prove to be the most token-efficient, offering dual benefits of improved accuracy and reduced computational cost.

In contrast, prioritizing maximum structural coverage (“Max Line Coverage”), a common heuristic in traditional bug detection, proved less effective for LLM refinement.
This suggests that while high-coverage inputs might explore more code paths, their potential complexity can overwhelm the LLM or obscure the specific fault, hindering effective repair.
This finding aligns with delta debugging principles [[27](https://arxiv.org/html/2506.18315v1#bib.bib27)], which advocate for identifying the simplest input that still triggers a failure.
We observe that LLMs, much like human developers, benefit from such minimized inputs.
Overly complex inputs can present convoluted information, potentially causing LLMs to get “lost in the middle” [[26](https://arxiv.org/html/2506.18315v1#bib.bib26)] and fail to discern the core deficiencies.
In contrast, the shortest input (or one with minimal runtime) that manifests a property violation provides a concise, focused fault signal.
This aids the LLM’s error localization and understanding, akin to how humans use simplified examples for effective debugging.
Consequently, based on these findings, PGS adopts the strategy of formulating feedback using the shortest input that triggers a property violation.

### IV-D RQ3: The Viability and Impact of LLM-Generated Property

![Refer to caption](x6.png)

Figure 6: 
Comparison of code generation outcome distributions (%) on LiveCodeBench with DeepSeek-R1-Distilled-32B. Categories include Pass, Runtime Error (incl. property violations), Wrong Answer, and TLE.

This research question investigates two key aspects: first, the LLM’s effectiveness in generating the PBT artifactsthat underpin PGS’s validation process, and second, how integrating these LLM-generated artifacts impacts the final distribution of code generation outcomes.

Generating Validation Artifacts is Easier for LLMs.
[Table IV](https://arxiv.org/html/2506.18315v1#S4.T4 "In IV-D RQ3: The Viability and Impact of LLM-Generated Property ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation") illuminates the LLM’s proficiency in generating PBT artifacts.
It compares the “Direct Pass” rate (LLM can generate correct code once without any refinement) with “Validation Gen. Acc.” (the accuracy of the LLM in formulating the validation artifacts or properties intended to check correctness).
We find that LLMs demonstrate considerably higher accuracy in generating these PBT artifacts than in directly producing correct code.
This suggests that conceptualizing and defining correctness criteria, even if focused on specific aspects, is a more tractable task for LLMs than generating a complete, error-free implementation from scratch, particularly for complex problems.

Integrating Properties Significantly Improves Outcomes.
While high ‘Validation Gen. Acc.” indicates LLM proficiency in formulating properties, the nature of these generated properties can vary; they might capture overarching invariants or focus on more specific, partial aspects of the problem’s requirements. It is often challenging to ascertain upfront whether a generated property provides “complete” validation versus “partial” validation concerning all desired behaviors.
Nevertheless, the practical utility of integrating these LLM-generated properties within PGS is clearly demonstrated by the shift in code generation outcomes, as shown in [Figure 6](https://arxiv.org/html/2506.18315v1#S4.F6 "In IV-D RQ3: The Viability and Impact of LLM-Generated Property ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation").
Despite potential variations in property scope, their integration via PGS significantly improves outcome distribution. “Wrong Answer” outcomes drop sharply from 25.3% (without refinement) to 10.5% with PGS. While “Runtime Errors” (including PBT assertion failures) increase from 4.6% to 11.8%, this reflects PGS converting latent logical flaws into explicit, actionable property violations, ultimately boosting “Pass” rates.

Thus, PGS effectively leverages the LLM’s greater aptitude for formulating these specification-derived properties. This process transforms elusive “Wrong Answers” into more structured, actionable feedback through property violations, demonstrating a practical pathway to improved code reliability by productively utilizing LLM-generated validation artifacts, regardless of whether they represent “complete” validation or “partial” validation.

TABLE IV: 
Comparison of Direct Pass Rates and Validation Generation Accuracy by Task Difficulty

| Difficulty | Direct Pass | Validation Gen. Acc. |
| --- | --- | --- |
| Easy | 62.4% | 82.4%(+20.0%) |
| Medium | 17.5% | 62.8%(+45.3%) |
| Hard | 1.1% | 48.9%(+47.8%) |

### IV-E RQ4: Performance Across Task Difficulty and LLMs

TABLE V: 
Outcome distribution across task difficulties

| Difficulty | Pass Tvsubscript𝑇𝑣T\_{v}italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT | Pass Thsubscript𝑇ℎT\_{h}italic\_T start\_POSTSUBSCRIPT italic\_h end\_POSTSUBSCRIPT | WA | RE | TLE |
| --- | --- | --- | --- | --- | --- |
| Easy | 65.9% | 62.4% | 31.9% | 5.4% | 0.3% |
| Medium | 33.2% | 17.5% | 61.6% | 12.5% | 8.4% |
| Hard | 18.1% | 1.1% | 68.9% | 17.6% | 12.4% |

Task Difficulty.
We examine the general challenges posed by varying task difficulty on LiveCodeBench, using DeepSeek-Coder-V2 as a representative example ([Table V](https://arxiv.org/html/2506.18315v1#S4.T5 "In IV-E RQ4: Performance Across Task Difficulty and LLMs ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation")).
A critical observation is that Wrong Answer (WA) is the predominant error type across all difficulty levels, indicating LLMs frequently produce syntactically correct but semantically flawed code.
Furthermore, the proportion of programs that pass visible public tests Tvsubscript𝑇𝑣T\_{v}italic\_T start\_POSTSUBSCRIPT italic\_v end\_POSTSUBSCRIPT yet ultimately fail hidden tests increases substantially with difficulty.
This growing discrepancy from easy to hard underscores the diminishing reliability of public tests alone for complex problems, highlighting the crucial need for more comprehensive test feedback.
PGS’s advantage becomes particularly pronounced on Hard tasks.
For instance, with DeepSeek-R1-Distilled-32B, PGS achieves a pass@1 of 40.7% on these challenging problems, substantially outperforming direct prompting (28.1%).
This efficacy on complex tasks stems from PGS’s PBT-centric approach.
We observe that even when generating a fully correct solution is exceedingly difficult, defining and verifying specific properties of a correct solution is often more tractable.
By prompting the Tester to establish such specification-derived properties, PGS guides refinement using these attainable correctness criteria, effectively uncovering critical flaws even when overall problem complexity is high.

Model Generalizability.
Against this backdrop of increasing challenge and public test limitations, PGS demonstrates consistent superiority across different LLMs, as detailed in [Table II](https://arxiv.org/html/2506.18315v1#S4.T2 "In IV-B RQ1: Overall Performance ‣ IV EXPERIMENTS AND RESULTS ‣ Use Property-Based Testing to Bridge LLM Code Generation and Validation").
Across all three LLMs, PGS achieves the highest overall pass@1 scores, surpassing other baseline relying on prompting techniques or TDD mechanism.
It indicates its benefits generalize across models of varying capabilities.

In summary, PGS performs robustly across different LLMs and excels as task difficulty increases, especially on Hard problems.
Its ability to generate targeted, property-driven feedback, proves crucial for guiding LLMs towards correct solutions in complex scenarios.

### IV-F Threats to Validity

We discuss several potential threats to our findings’ validity.

* •

  Generalizability:
  To mitigate this, our evaluation used three diverse benchmarks and three distinct LLMs.
  PGS’s consistent strong performance across these settings supports its effectiveness.
  Future work could explore broader LLM variety, multi-language [[38](https://arxiv.org/html/2506.18315v1#bib.bib38)] contexts, and different task types.
* •

  Data Leakage:
  Pretraining data for the LLMs might include benchmark samples, potentially inflating absolute scores.
  However, as all methods used the same LLMs, this affects baselines and PGS comparably, preserving the integrity of our relative performance comparisons and conclusions about PGS’s advantages.
* •

  PBT Artifact Quality:
  PGS’s efficacy depends on the quality of LLM-generated properties.
  Trivial or irrelevant properties may limit PBT’s benefits.
* •

  Hyperparameters:
  Specific settings (*e.g.*, iteration counts, temperature, prompt phrasing) could influence outcomes.
  We used common settings and consistent budgets for fairness.

While we validate properties and explore effective feedback, ensuring insightful properties across diverse problems remains a challenge.
Future work could focus on enhancing property generation.
On the other hand, for optimal feedback formation, exhaustive tuning of all parameters is beyond this study’s scope.
We expect PGS’s core PBT-driven decoupling mechanism to yield benefits across reasonable configurations.

## V RELATED WORK

Test-Driven Code Generation with LLMs.
LLMs have made remarkable strides in code generation. Models like ChatGPT [[1](https://arxiv.org/html/2506.18315v1#bib.bib1)], Qwen [[39](https://arxiv.org/html/2506.18315v1#bib.bib39)], Llama [[40](https://arxiv.org/html/2506.18315v1#bib.bib40)], and DeepSeek [[41](https://arxiv.org/html/2506.18315v1#bib.bib41)], trained on extensive text and code corpora, can generate code snippets for diverse programming tasks.
Approaches such as planning algorithms [[28](https://arxiv.org/html/2506.18315v1#bib.bib28), [42](https://arxiv.org/html/2506.18315v1#bib.bib42), [43](https://arxiv.org/html/2506.18315v1#bib.bib43)] and multi-agent collaboration frameworks [[44](https://arxiv.org/html/2506.18315v1#bib.bib44), [23](https://arxiv.org/html/2506.18315v1#bib.bib23), [45](https://arxiv.org/html/2506.18315v1#bib.bib45)] have been developed to enhance the quality of generated code. However, despite these efforts, the generated code often contains errors, that undermine reliability.
To address these errors, a key research stream [[46](https://arxiv.org/html/2506.18315v1#bib.bib46), [47](https://arxiv.org/html/2506.18315v1#bib.bib47), [48](https://arxiv.org/html/2506.18315v1#bib.bib48)] focuses on emulating human software development workflows by providing external error feedback to guide LLMs in iterative code refinement.
Existing methods range from direct error feedback prompting [[35](https://arxiv.org/html/2506.18315v1#bib.bib35)] to multi-step debugging pipelines that integrate static analysis and debuggers [[6](https://arxiv.org/html/2506.18315v1#bib.bib6)].
These methods consistently outperform the baseline models, demonstrating that external error feedback significantly enhances LLMs’ code generation capability.
However, their effectiveness critically depends on the availability of high-quality test cases. In many real-world scenarios, the scarcity of test cases renders these approaches inapplicable.
This paper focuses on bridging this gap by developing methods to generate usable extra test cases specifically for Test-Driven Code Generation.

Test Input Metrics.
The efficacy of generated test inputs is often assessed using various metrics. Widely adopted are structural coverage criteria, such as line or branch coverage, which quantify the extent to which inputs exercise program code [[49](https://arxiv.org/html/2506.18315v1#bib.bib49)].
While indicative, high structural coverage does not guarantee comprehensive fault detection. Consequently, more fault-oriented metrics, like mutation scores that evaluate a test suite’s ability to identify seeded defects [[50](https://arxiv.org/html/2506.18315v1#bib.bib50), [51](https://arxiv.org/html/2506.18315v1#bib.bib51), [52](https://arxiv.org/html/2506.18315v1#bib.bib52)], are also considered crucial for gauging deeper testing quality.
For Property-Based Testing, while not always explicitly measured by these traditional metrics during its dynamic input generation, the implicit quality of its inputs lies in their power to efficiently find counterexamples that violate specified properties [[53](https://arxiv.org/html/2506.18315v1#bib.bib53)], often through diverse and boundary-value exploration [[17](https://arxiv.org/html/2506.18315v1#bib.bib17)].
However, these traditional metrics primarily focus on fault detection efficacy and generally do not consider the characteristics of feedback suitable for guiding Large Language Models in code refinement. Our work specifically investigates what makes PBT-derived feedback effective for LLMs.

LLM-based Test Generation.
Automated test generation plays an increasingly crucial role in ensuring the reliability of code produced by Large Language Models (LLMs)[[54](https://arxiv.org/html/2506.18315v1#bib.bib54)] and in facilitating effective iterative refinement processes[[44](https://arxiv.org/html/2506.18315v1#bib.bib44)].
While contemporary LLMs, sometimes building upon earlier fine-tuning efforts on specific datasets [[55](https://arxiv.org/html/2506.18315v1#bib.bib55)], can directly generate numerous input-output test assertions (exemplified by methods like CodeT [[11](https://arxiv.org/html/2506.18315v1#bib.bib11)], CodeCoT [[56](https://arxiv.org/html/2506.18315v1#bib.bib56)], and AID [[12](https://arxiv.org/html/2506.18315v1#bib.bib12)]), such generated tests inherently risk perpetuating the “cycle of self-deception”.
If these tests are derived from a flawed initial interpretation of the problem specification by the LLM, they may offer limited semantic feedback and fail to expose critical errors.
Alternatively, prompting LLMs (often via CoT) to predict test outputs for oracle construction [[10](https://arxiv.org/html/2506.18315v1#bib.bib10)] can be more challenging than direct code generation for complex problems [[16](https://arxiv.org/html/2506.18315v1#bib.bib16)], and is often inefficient due to per-input LLM calls, leading to low oracle accuracy.
These limitations highlight the need for more robust, independent test generation. Our framework, PGS, distinctively addresses this by operationalizing Property-Based Testing [[18](https://arxiv.org/html/2506.18315v1#bib.bib18)].
In PGS, LLMs formulate specification-derived properties that guide dynamic input generation, effectively breaking the “cycle of self-deception” and providing abstract, actionable feedback crucial for robust code refinement.

## VI CONCLUSION

In this paper, we proposed PGS, a novel framework that, to our knowledge, is the first to systematically integrate Property-Based Testing as the core engine for iterative refinement in LLM-based code generation.
Our approach, centered around collaborative Generator and Tester agents, demonstrates that anchoring generating in easily verifiable properties, rather than direct test oracle prediction, significantly enhances the correctness of generated code.
Extensive experiments on diverse benchmarks and LLMs demonstrate the superior accuracy and robustness of PGS compared to existing approaches.
Our work represents a significant step towards more reliable automated code generation by leveraging principled testing methodologies to effectively guide and validate LLM outputs.

## References

* [1]

  R. OpenAI, “Gpt-4 technical report. arxiv 2303.08774,” *View in Article*, vol. 2, no. 5, 2023.
* [2]

  J. Bai, S. Bai, Y. Chu, Z. Cui, K. Dang, X. Deng, Y. Fan, W. Ge, Y. Han, F. Huang *et al.*, “Qwen technical report,” *arXiv preprint arXiv:2309.16609*, 2023.
* [3]

  Q. Zhu, D. Guo, Z. Shao, D. Yang, P. Wang, R. Xu, Y. Wu, Y. Li, H. Gao, S. Ma *et al.*, “Deepseek-coder-v2: Breaking the barrier of closed-source models in code intelligence,” *arXiv preprint arXiv:2406.11931*, 2024.
* [4]

  J. Liu, C. S. Xia, Y. Wang, and L. Zhang, “Is your code generated by chatgpt really correct? rigorous evaluation of large language models for code generation,” *Advances in Neural Information Processing Systems*, vol. 36, 2024.
* [5]

  S. Jiang, Y. Wang, and Y. Wang, “Selfevolve: A code evolution framework via large language models,” *arXiv preprint arXiv:2306.02907*, 2023.
* [6]

  L. Zhong, Z. Wang, and J. Shang, “Debug like a human: A large language model debugger via verifying runtime execution step by step,” in *Findings of the Association for Computational Linguistics ACL 2024*, 2024, pp. 851–870.
* [7]

  N. Shinn, F. Cassano, A. Gopinath, K. Narasimhan, and S. Yao, “Reflexion: Language agents with verbal reinforcement learning,” *Advances in Neural Information Processing Systems*, vol. 36, 2024.
* [8]

  M. Kazemitabaar, J. Chow, C. K. T. Ma, B. J. Ericson, D. Weintrop, and T. Grossman, “Studying the effect of ai code generators on supporting novice learners in introductory programming,” in *Proceedings of the 2023 CHI Conference on Human Factors in Computing Systems*, 2023, pp. 1–23.
* [9]

  M. Wermelinger, “Using github copilot to solve simple programming problems,” in *Proceedings of the 54th ACM Technical Symposium on Computer Science Education V. 1*, 2023, pp. 172–178.
* [10]

  K. Li and Y. Yuan, “Large language models as test case generators: Performance evaluation and enhancement,” *arXiv preprint arXiv:2404.13340*, 2024.
* [11]

  B. Chen, F. Zhang, A. Nguyen, D. Zan, Z. Lin, J.-G. Lou, and W. Chen, “Codet: Code generation with generated tests,” in *The Eleventh International Conference on Learning Representations*, 2023.
* [12]

  K. Liu, Y. Liu, Z. Chen, J. M. Zhang, Y. Han, Y. Ma, G. Li, and G. Huang, “Llm-powered test case generation for detecting tricky bugs,” *arXiv preprint arXiv:2404.10304*, 2024.
* [13]

  M. Schäfer, S. Nadi, A. Eghbali, and F. Tip, “An empirical evaluation of using large language models for automated unit test generation,” *IEEE Transactions on Software Engineering*, vol. 50, no. 1, pp. 85–105, 2023.
* [14]

  E. T. Barr, M. Harman, P. McMinn, M. Shahbaz, and S. Yoo, “The oracle problem in software testing: A survey,” *IEEE transactions on software engineering*, vol. 41, no. 5, pp. 507–525, 2014.
* [15]

  S. B. Hossain and M. Dwyer, “Togll: Correct and strong test oracle generation with llms,” *arXiv preprint arXiv:2405.03786*, 2024.
* [16]

  N. Jain, K. Han, A. Gu, W.-D. Li, F. Yan, T. Zhang, S. Wang, A. Solar-Lezama, K. Sen, and I. Stoica, “Livecodebench: Holistic and contamination free evaluation of large language models for code,” in *The Thirteenth International Conference on Learning Representations*, 2025.
* [17]

  K. Claessen and J. Hughes, “Quickcheck: a lightweight tool for random testing of haskell programs,” in *Proceedings of the fifth ACM SIGPLAN international conference on Functional programming*, 2000, pp. 268–279.
* [18]

  V. Vikram, C. Lemieux, J. Sunshine, and R. Padhye, “Can large language models write good property-based tests?” *arXiv preprint arXiv:2307.04346*, 2023.
* [19]

  M. Chen, J. Tworek, H. Jun, Q. Yuan, H. P. D. O. Pinto, J. Kaplan, H. Edwards, Y. Burda, N. Joseph, G. Brockman *et al.*, “Evaluating large language models trained on code,” *arXiv preprint arXiv:2107.03374*, 2021.
* [20]

  J. Austin, A. Odena, M. Nye, M. Bosma, H. Michalewski, D. Dohan, E. Jiang, C. Cai, M. Terry, Q. Le *et al.*, “Program synthesis with large language models,” *arXiv preprint arXiv:2108.07732*, 2021.
* [21]

  N. S. Mathews and M. Nagappan, “Test-driven development and llm-based code generation,” in *Proceedings of the 39th IEEE/ACM International Conference on Automated Software Engineering*, ser. ASE ’24.   Association for Computing Machinery, 2024, p. 1583–1594.
* [22]

  M. Suzgun, N. Scales, N. Schärli, S. Gehrmann, Y. Tay, H. W. Chung, A. Chowdhery, Q. V. Le, E. H. Chi, D. Zhou *et al.*, “Challenging big-bench tasks and whether chain-of-thought can solve them,” *arXiv preprint arXiv:2210.09261*, 2022.
* [23]

  Z. Xi, W. Chen, X. Guo, W. He, Y. Ding, B. Hong, M. Zhang, J. Wang, S. Jin, E. Zhou *et al.*, “The rise and potential of large language model based agents: A survey,” *Science China Information Sciences*, vol. 68, no. 2, p. 121101, 2025.
* [24]

  D. Guo, D. Yang, H. Zhang, J. Song, R. Zhang, R. Xu, Q. Zhu, S. Ma, P. Wang, X. Bi *et al.*, “Deepseek-r1: Incentivizing reasoning capability in llms via reinforcement learning,” *arXiv preprint arXiv:2501.12948*, 2025.
* [25]

  A. El-Kishky, A. Wei, A. Saraiva, B. Minaiev, D. Selsam, D. Dohan, F. Song, H. Lightman, I. Clavera, J. Pachocki *et al.*, “Competitive programming with large reasoning models,” *arXiv preprint arXiv:2502.06807*, 2025.
* [26]

  N. F. Liu, K. Lin, J. Hewitt, A. Paranjape, M. Bevilacqua, F. Petroni, and P. Liang, “Lost in the middle: How language models use long contexts,” *Transactions of the Association for Computational Linguistics*, vol. 12, 2024.
* [27]

  G. Misherghi and Z. Su, “Hdd: hierarchical delta debugging,” in *Proceedings of the 28th international conference on Software engineering*, 2006, pp. 142–151.
* [28]

  H. Zhang, W. Cheng, Y. Wu, and W. Hu, “A pair programming framework for code generation via multi-plan exploration and feedback-driven refinement,” in *The 39th IEEE/ACM International Conference on Automated Software Engineering (ASE 2024)*, 2024.
* [29]

  Y. Li, D. Choi, J. Chung, N. Kushman, J. Schrittwieser, R. Leblond, T. Eccles, J. Keeling, F. Gimeno, A. Dal Lago *et al.*, “Competition-level code generation with alphacode,” *Science*, vol. 378, no. 6624, pp. 1092–1097, 2022.
* [30]

  A. Ni, S. Iyer, D. Radev, V. Stoyanov, W.-t. Yih, S. Wang, and X. V. Lin, “Lever: Learning to verify language-to-code generation with execution,” in *International Conference on Machine Learning*.   PMLR, 2023, pp. 26 106–26 128.
* [31]

  H. Yu, B. Shen, D. Ran, J. Zhang, Q. Zhang, Y. Ma, G. Liang, Y. Li, Q. Wang, and T. Xie, “Codereval: A benchmark of pragmatic code generation with generative pre-trained models,” in *Proceedings of the 46th IEEE/ACM International Conference on Software Engineering*, 2024, pp. 1–12.
* [32]

  M. Yasunaga and P. Liang, “Break-it-fix-it: Unsupervised learning for program repair,” in *International conference on machine learning*.   PMLR, 2021, pp. 11 941–11 952.
* [33]

  B. Hui, J. Yang, Z. Cui, J. Yang, D. Liu, L. Zhang, T. Liu, J. Zhang, B. Yu, K. Lu *et al.*, “Qwen2. 5-coder technical report,” *arXiv preprint arXiv:2409.12186*, 2024.
* [34]

  J. Wei, X. Wang, D. Schuurmans, M. Bosma, F. Xia, E. Chi, Q. V. Le, D. Zhou *et al.*, “Chain-of-thought prompting elicits reasoning in large language models,” *Advances in neural information processing systems*, vol. 35, pp. 24 824–24 837, 2022.
* [35]

  K. Zhang, Z. Li, J. Li, G. Li, and Z. Jin, “Self-edit: Fault-aware code editor for code generation,” *arXiv preprint arXiv:2305.04087*, 2023.
* [36]

  Y. Shi, S. Wang, C. Wan, and X. Gu, “From code to correctness: Closing the last mile of code generation with hierarchical debugging,” *arXiv preprint arXiv:2410.01215*, 2024.
* [37]

  X. Chen, M. Lin, N. Schärli, and D. Zhou, “Teaching large language models to self-debug,” *arXiv preprint arXiv:2304.05128*, 2023.
* [38]

  Q. Zheng, X. Xia, X. Zou, Y. Dong, S. Wang, Y. Xue, L. Shen, Z. Wang, A. Wang, Y. Li *et al.*, “Codegeex: A pre-trained model for code generation with multilingual benchmarking on humaneval-x,” in *Proceedings of the 29th ACM SIGKDD Conference on Knowledge Discovery and Data Mining*, 2023, pp. 5673–5684.
* [39]

  A. Yang, A. Li, B. Yang, B. Zhang, B. Hui, B. Zheng, B. Yu, C. Gao, C. Huang, C. Lv *et al.*, “Qwen3 technical report,” *arXiv preprint arXiv:2505.09388*, 2025.
* [40]

  H. Touvron, T. Lavril, G. Izacard, X. Martinet, M.-A. Lachaux, T. Lacroix, B. Rozière, N. Goyal, E. Hambro, F. Azhar *et al.*, “Llama: Open and efficient foundation language models,” *arXiv preprint arXiv:2302.13971*, 2023.
* [41]

  A. Liu, B. Feng, B. Xue, B. Wang, B. Wu, C. Lu, C. Zhao, C. Deng, C. Zhang, C. Ruan *et al.*, “Deepseek-v3 technical report,” *arXiv preprint arXiv:2412.19437*, 2024.
* [42]

  J. Li, G. Li, Y. Li, and Z. Jin, “Structured chain-of-thought prompting for code generation,” *ACM Transactions on Software Engineering and Methodology*, vol. 34, no. 2, pp. 1–23, 2025.
* [43]

  X. Jiang, Y. Dong, L. Wang, Z. Fang, Q. Shang, G. Li, Z. Jin, and W. Jiao, “Self-planning code generation with large language models,” *ACM Transactions on Software Engineering and Methodology*, vol. 33, no. 7, pp. 1–30, 2024.
* [44]

  D. Huang, Q. Bu, J. M. Zhang, M. Luck, and H. Cui, “Agentcoder: Multi-agent-based code generation with iterative testing and optimisation,” *arXiv preprint arXiv:2312.13010*, 2023.
* [45]

  Y. Dong, X. Jiang, Z. Jin, and G. Li, “Self-collaboration code generation via chatgpt,” *ACM Transactions on Software Engineering and Methodology*, vol. 33, no. 7, pp. 1–38, 2024.
* [46]

  H. Jin, L. Huang, H. Cai, J. Yan, B. Li, and H. Chen, “From llms to llm-based agents for software engineering: A survey of current, challenges and future,” *arXiv preprint arXiv:2408.02479*, 2024.
* [47]

  S. B. Hossain, N. Jiang, Q. Zhou, X. Li, W.-H. Chiang, Y. Lyu, H. Nguyen, and O. Tripp, “A deep dive into large language models for automated bug localization and repair,” *Proceedings of the ACM on Software Engineering*, vol. 1, no. FSE, pp. 1471–1493, 2024.
* [48]

  C. S. Xia and L. Zhang, “Conversational automated program repair,” *arXiv preprint arXiv:2301.13246*, 2023.
* [49]

  I. Sillitoe, N. Bordin, N. Dawson, V. P. Waman, P. Ashford, H. M. Scholes, C. S. Pang, L. Woodridge, C. Rauer, N. Sen *et al.*, “Cath: increased structural coverage of functional space,” *Nucleic acids research*, vol. 49, no. D1, pp. D266–D273, 2021.
* [50]

  Y. Jia and M. Harman, “An analysis and survey of the development of mutation testing,” *IEEE transactions on software engineering*, vol. 37, no. 5, pp. 649–678, 2010.
* [51]

  M. Papadakis, M. Kintis, J. Zhang, Y. Jia, Y. Le Traon, and M. Harman, “Mutation testing advances: an analysis and survey,” in *Advances in computers*.   Elsevier, 2019, vol. 112, pp. 275–378.
* [52]

  J. Zhang, Z. Wang, L. Zhang, D. Hao, L. Zang, S. Cheng, and L. Zhang, “Predictive mutation testing,” in *Proceedings of the 25th international symposium on software testing and analysis*, 2016, pp. 342–353.
* [53]

  X. Yang, Y. Chen, E. Eide, and J. Regehr, “Finding and understanding bugs in c compilers,” in *Proceedings of the 32nd ACM SIGPLAN conference on Programming language design and implementation*, 2011, pp. 283–294.
* [54]

  X.-Y. Li, J.-T. Xue, Z. Xie, and M. Li, “Think outside the code: Brainstorming boosts large language models in code generation,” *arXiv preprint arXiv:2305.10679*, 2023.
* [55]

  M. Tufano, D. Drain, A. Svyatkovskiy, S. K. Deng, and N. Sundaresan, “Unit test case generation with transformers and focal context,” *arXiv preprint arXiv:2009.05617*, 2020.
* [56]

  D. Huang, Q. Bu, Y. Qing, and H. Cui, “Codecot: Tackling code syntax errors in cot reasoning for code generation,” *CoRR*, vol. 2308, pp. 1–20, 2023.

Generated on Mon Jun 23 05:54:25 2025 by [LaTeXML![Mascot Sammy](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAOCAYAAAD5YeaVAAAAAXNSR0IArs4c6QAAAAZiS0dEAP8A/wD/oL2nkwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB9wKExQZLWTEaOUAAAAddEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIFRoZSBHSU1Q72QlbgAAAdpJREFUKM9tkL+L2nAARz9fPZNCKFapUn8kyI0e4iRHSR1Kb8ng0lJw6FYHFwv2LwhOpcWxTjeUunYqOmqd6hEoRDhtDWdA8ApRYsSUCDHNt5ul13vz4w0vWCgUnnEc975arX6ORqN3VqtVZbfbTQC4uEHANM3jSqXymFI6yWazP2KxWAXAL9zCUa1Wy2tXVxheKA9YNoR8Pt+aTqe4FVVVvz05O6MBhqUIBGk8Hn8HAOVy+T+XLJfLS4ZhTiRJgqIoVBRFIoric47jPnmeB1mW/9rr9ZpSSn3Lsmir1fJZlqWlUonKsvwWwD8ymc/nXwVBeLjf7xEKhdBut9Hr9WgmkyGEkJwsy5eHG5vN5g0AKIoCAEgkEkin0wQAfN9/cXPdheu6P33fBwB4ngcAcByHJpPJl+fn54mD3Gg0NrquXxeLRQAAwzAYj8cwTZPwPH9/sVg8PXweDAauqqr2cDjEer1GJBLBZDJBs9mE4zjwfZ85lAGg2+06hmGgXq+j3+/DsixYlgVN03a9Xu8jgCNCyIegIAgx13Vfd7vdu+FweG8YRkjXdWy329+dTgeSJD3ieZ7RNO0VAXAPwDEAO5VKndi2fWrb9jWl9Esul6PZbDY9Go1OZ7PZ9z/lyuD3OozU2wAAAABJRU5ErkJggg==)](http://dlmf.nist.gov/LaTeXML/)
