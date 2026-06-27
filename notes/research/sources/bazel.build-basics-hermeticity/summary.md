# Summary

- **Title:** Hermeticity | Bazel
- **Authors:** Bazel project / Google (no individual authors; documentation page)
- **URL:** https://bazel.build/basics/hermeticity
- **Date:** Page last updated 2026-05-07 UTC (captured 2026-06-27)
- **Venue:** Bazel official documentation site (Concepts / Getting started)
- **Source type:** (vendor documentation)

## What the source claims

Defines **hermeticity** for build systems and lists the benefits of hermetic builds plus
strategies for finding and fixing non-hermetic behavior. Core claim: a hermetic build
system, given the same source and configuration, always produces the same output by
isolating the build from the host system.

Verbatim quotes:

> "When given the same input source code and product configuration, a hermetic build system
> always returns the same output by isolating the build from changes to the host system."

> "In order to isolate the build, hermetic builds are insensitive to libraries and other
> software installed on the local or remote host machine. They depend on specific versions
> of build tools, such as compilers, and dependencies, such as libraries."

On the two aspects of hermeticity:

> "**Isolation**: Hermetic build systems treat tools as source code. They download copies of
> tools and manage their storage and use inside managed file trees."

> "**Source identity**: Hermetic build systems try to ensure the sameness of inputs. ...
> Hermetic build systems use this hash to identify changes to the build's input."

On troubleshooting:

> "Ensure null sequential builds: If you run `make` and get a successful build, running the
> build again should not rebuild any targets. If you run each build step twice or on
> different systems, compare a hash of the file contents and get results that differ, the
> build is not reproducible."

## Method / evidence type

Vendor conceptual documentation — definitions, benefit lists, and practical guidance. No
study, data, or measurements. Stated benefits of hermetic builds: **Speed** (cacheable
actions), **Parallel execution** (action graph), **Multiple builds** (different tool
versions on one machine), and **Reproducibility** (troubleshooting).

Listed common sources of non-hermeticity: arbitrary processing in `.mk` files;
non-deterministic file creation (build IDs/timestamps); host-varying system binaries
(`/usr/bin`, absolute paths, system C++ compilers); writing to the source tree during the
build. Listed troubleshooting steps: null sequential builds, debugging local cache hits
across machines, building inside a Docker container with only the source tree + explicit
host tools, remote execution rules, strict per-action sandboxing, and the
`--experimental_workspace_rules_log_file=PATH` flag to log potentially non-hermetic
workspace-rule actions.

## Numbers recorded

None. The page contains no quantitative data, benchmarks, or measurements (only Bazel
version links and a "Last updated 2026-05-07 UTC" timestamp).

## Scope, limitations, and gaps

- Marketing/educational documentation, not an empirical or peer-reviewed source; claims are
  asserted, not evidenced.
- Bazel-specific framing; the "Hermeticity with Bazel" section is a list of external
  BazelCon talk links (SpaceX, Uber/TwoSigma, IBM, BMW, GM Cruise, etc.), not analysis.
- Content is licensed CC BY 4.0 (code samples Apache 2.0).

## Capture status

`transcript.md` is the full rendered documentation page (`capture_status: ok`, re-extract),
including Overview, Benefits, Identifying non-hermeticity, Troubleshooting, and Hermeticity
with Bazel sections, plus site navigation chrome. No content was blocked.
