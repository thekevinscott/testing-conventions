# Summary

- **Title:** SandboxEval: Towards Securing Test Environment for Untrusted Code
- **Authors:** Rafiqul Rabin, Jesse Hostetler, Sean McGregor, Brett Weir, Nick Judd (Digital Safety Research Institute, UL Research Institutes)
- **URL:** https://arxiv.org/pdf/2504.00018
- **Date:** arXiv v1, 27 Mar 2025 (cs.CR)
- **Venue:** Not stated on the captured PDF (arXiv preprint, arXiv:2504.00018v1)
- **Source type:** (arXiv preprint; full paper PDF captured)

## What the source claims

Presents **SandboxEval**, a test suite of manually crafted test cases that simulate
real-world safety scenarios for environments executing untrusted LLM-generated code. The
suite probes whether a sandbox protects against sensitive-information exposure, filesystem
manipulation, external communication, and other dangerous operations. The authors argue
the suite belongs in the "scoring"/measurement step of LLM assessment frameworks, and
demonstrate it on a running instance of **Dyff**, an open-source AI assessment framework.

Verbatim quotes:

> "SandboxEval tests 51 properties associated with malicious and potentially harmful code
> execution scenarios, such as sensitive information exposure, filesystem manipulation, and
> external communication."

> "The only assumption is that it is probing the security of a Linux system."

> "We found that LLM-generated test cases often contain unusable code with outcomes that
> were difficult to assess, and, for these and related reasons, we continued the analysis
> using our set of hand-crafted test cases."

> "We found that, on average, only 16.47% of the LLM-generated code was syntactically valid
> based on the test descriptions used as queries."

> "a recent industry survey found that 76% of all Docker containers are running with
> elevated privileges."

## Method / evidence type

- **Artifact construction**: an interdisciplinary team (ML researchers, software engineers,
  infrastructure experts) hand-wrote a Python test case for each of 51 test descriptions
  (Tables I–III), grouped into Expose System/Directory/Metadata, Manipulate
  Structure/Content/Privilege, External Communications, and Dangerous Operations.
- **LLM test-generation attempt**: prompted three CodeLlama-7B variants (7B, 7B-Python,
  7B-Instruct, `max_tokens=400`) to auto-generate candidate test cases; deferred this to
  future work because outputs were largely unusable.
- **Case study / deployment**: executed the hand-crafted suite inside a real Dyff instance
  on Kubernetes/gVisor with a deny-all NetworkPolicy; each test case classified **Accessed**,
  **Denied**, or **Unknown** (Table IV). Cautious execution used proxy operations rather
  than truly destructive actions; some tests also run on a "research laptop" for contrast.

## Numbers recorded

- **51** total tests/properties/scenarios. Per-category counts: 10 system, 6 directory, 3
  metadata (Table I); 7 structure, 6 content, 4 privilege (Table II); 8 external
  communications, 7 dangerous operations (Table III).
- LLM generation: **10** candidate cases per scenario, **500+** prompts per model;
  syntactic validity averaged **16.47%** (CodeLlama-7B-Python highest at **27.8%**); after
  stripping non-code text, validity rose to **40.13%** average (Python variant **45.0%**).
- Dyff filesystem exploration to a recursive depth of **10** levels: **527** readable files
  from **136** directories, **5,901** writable files from **1,272** directories, **7,122**
  executable files from **2,407** directories. Only **/proc** and **/tmp** were writable.
- Dyff results (Table IV): External Communications all **Denied** (Ping, DNS, HTTP, FTP,
  SSH, SMTP, messaging/cloud); Dangerous Operations all **Denied**; several Expose-System
  cases **Accessed** (Platform, CPU, Memory, Disk, Network, Locale/Time, Environment
  Variables), with Sensor/User/PID **Denied**.
- Related-work figures cited: ~**40%** of Copilot programs vulnerable (Pearce et al.); up to
  **27.25%** insecure in a Copilot replication (Majdinasab et al.); **32.8%** of Copilot
  snippets had security issues (Fu et al.); Copilot replicates human-introduced
  vulnerabilities ~**33%** of the time (Asare et al.).

## Scope, limitations, and gaps

- Authors' stated threats to validity: suite is **not exhaustive** (51 scenarios are a
  representative, not complete, sample of malicious activity); evaluated on **one** platform
  (Dyff, Linux/Kubernetes) so results may vary on other deployments; implemented **entirely
  in Python**, so other languages may surface different vulnerabilities; LLM-based test
  generation was attempted but outputs were not automatically usable.
- The Dyff demonstration tests a single configuration; "Failures on some of these tests may
  not be relevant, depending on deployment details."
- Filesystem-manipulation tests were assessed by checking access rights (proxy), not by
  performing the destructive actions, so they indicate risk rather than confirmed exploit.

## Capture status

`transcript.md` is the full PDF text (`capture_status: ok`, curl), including abstract,
Sections I–VI, Tables I–IV (rendered as text), and the reference list. Figures/tables are
present as extracted text; the table layouts are linearized but legible.
