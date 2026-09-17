---
url: https://bazel.build/basics/hermeticity
title: Hermeticity  |  Bazel
fetched: 2026-06-27
raw: raw.html
transport: re-extract
capture_status: ok
---

[Skip to main content](#main-content)

[![Bazel](https://www.gstatic.com/devrel-devsite/prod/v11431966d26d9f049ef61662c2b798f1cdee8af320f1ba0f77a43eee64301d60/bazel/images/lockup.svg)](/)

[About Bazel](https://bazel.build/about)
[Getting started](https://bazel.build/start)
[User guide](https://bazel.build/docs)
[Reference](https://bazel.build/reference)
More

[Extending](https://bazel.build/extending)
[Community](https://bazel.build/community)
[Versioned docs](https://bazel.build/versions)

* [9.1.0](https://bazel.build/versions/9.1.0)
* [9.0.0](https://bazel.build/versions/9.0.0)
* [8.7.0](https://bazel.build/versions/8.7.0)
* [8.6](https://bazel.build/versions/8.6.0)
* [8.5](https://bazel.build/versions/8.5.0)
* [8.4](https://bazel.build/versions/8.4.0)
* [8.3](https://bazel.build/versions/8.3.0)
* [8.2](https://bazel.build/versions/8.2.0)
* [7.7](https://bazel.build/versions/7.7.0)
* [6.6](https://bazel.build/versions/6.6.0)

* [Nightly](https://bazel.build/)
* [More…](https://bazel.build/versions)

* [English](https://bazel.build/basics/hermeticity)
* [Español – América Latina](https://bazel.build/basics/hermeticity?hl=es-419)
* [Indonesia](https://bazel.build/basics/hermeticity?hl=id)
* [Português – Brasil](https://bazel.build/basics/hermeticity?hl=pt-br)
* [Tiếng Việt](https://bazel.build/basics/hermeticity?hl=vi)
* [Türkçe](https://bazel.build/basics/hermeticity?hl=tr)
* [עברית](https://bazel.build/basics/hermeticity?hl=he)
* [العربيّة](https://bazel.build/basics/hermeticity?hl=ar)
* [فارسی](https://bazel.build/basics/hermeticity?hl=fa)
* [हिंदी](https://bazel.build/basics/hermeticity?hl=hi)
* [বাংলা](https://bazel.build/basics/hermeticity?hl=bn)
* [ภาษาไทย](https://bazel.build/basics/hermeticity?hl=th)
* [中文 – 简体](https://bazel.build/basics/hermeticity?hl=zh-cn)
* [中文 – 繁體](https://bazel.build/basics/hermeticity?hl=zh-tw)
* [日本語](https://bazel.build/basics/hermeticity?hl=ja)
* [한국어](https://bazel.build/basics/hermeticity?hl=ko)
[GitHub](//github.com/bazelbuild/bazel/)

[Sign in](https://bazel.build/_d/signin?continue=https%3A%2F%2Fbazel.build%2Fbasics%2Fhermeticity&prompt=select_account)

* [Get started with Bazel quickly](https://bazel.build/start)

[Install](https://bazel.build/install)
[First build tutorials](https://bazel.build/start/cpp)
[Concepts](https://bazel.build/concepts/build-ref)
More

[![Bazel](https://www.gstatic.com/devrel-devsite/prod/v11431966d26d9f049ef61662c2b798f1cdee8af320f1ba0f77a43eee64301d60/bazel/images/lockup.svg)](/)

* [About Bazel](/about)
* [Getting started](/start)
  + [Install](/install)
  + [First build tutorials](/start/cpp)
  + [Concepts](/concepts/build-ref)
* [User guide](/docs)
* [Reference](/reference)
* [Extending](/extending)
* [Community](/community)
* [Versioned docs](/versions)
  + More
* [GitHub](//github.com/bazelbuild/bazel/)

* [Workspaces, packages, & targets](/concepts/build-ref)
* [Labels](/concepts/labels)
* [BUILD files](/concepts/build-files)
* [Dependencies](/concepts/dependencies)
* [Visibility](/concepts/visibility)
* [Platforms](/concepts/platforms)
* [Hermeticity](/basics/hermeticity)

* [9.1.0](/versions/9.1.0)
* [9.0.0](/versions/9.0.0)
* [8.7.0](/versions/8.7.0)
* [8.6](/versions/8.6.0)
* [8.5](/versions/8.5.0)
* [8.4](/versions/8.4.0)
* [8.3](/versions/8.3.0)
* [8.2](/versions/8.2.0)
* [7.7](/versions/7.7.0)
* [6.6](/versions/6.6.0)
* [Nightly](/)
* [More…](/versions)

* On this page
* [Overview](#overview)
* [Benefits](#benefits)
* [Identifying non-hermeticity](#nonhermeticity)
* [Troubleshooting non-hermetic builds](#troubleshooting-nonhermeticity)
* [Hermeticity with Bazel](#hermeticity-bazel)

The [Build Foundation](https://docs.google.com/presentation/d/14Yjm4gghabys0WrHgTNPlsaUmEWrgrSS_IEe2FJTzQ4/preview?slide=id.p) started enrolling founding members! Join the [mailing list](https://groups.linuxfoundation.org/g/build-foundation-formation/message/5), read the [participation agreement](https://cdn.platform.linuxfoundation.org/agreements/build-foundation.pdf), and [enroll](https://enrollment.lfx.linuxfoundation.org/?project=build-foundation)!

* [Bazel](https://bazel.build/)
* [Getting started](https://bazel.build/start)
* [Concepts](https://bazel.build/concepts/build-ref)

Was this helpful?

Send feedback

# Hermeticity Stay organized with collections Save and categorize content based on your preferences.

* On this page
* [Overview](#overview)
* [Benefits](#benefits)
* [Identifying non-hermeticity](#nonhermeticity)
* [Troubleshooting non-hermetic builds](#troubleshooting-nonhermeticity)
* [Hermeticity with Bazel](#hermeticity-bazel)

[Report an issueopen\_in\_new](https://github.com/bazelbuild/bazel/issues/new?title=%5Bbazel.build%5D+Problem+with+/basics/hermeticity&template=doc_issue.yml&link=https%3A%2F%2Fbazel.build/basics/hermeticity)
[View sourceopen\_in\_new](https://github.com/bazelbuild/bazel/tree/master/site/en/basics/hermeticity.md)

**Nightly**

·
[9.1](/versions/9.1.0/basics/hermeticity)
·
[9.0](/versions/9.0.0/basics/hermeticity)
·
[8.7](/versions/8.7.0/basics/hermeticity)
·
[8.6](/versions/8.6.0/basics/hermeticity)
·
[8.5](/versions/8.5.0/basics/hermeticity)
·
[8.4](/versions/8.4.0/basics/hermeticity)
·
[8.3](/versions/8.3.0/basics/hermeticity)
·
[8.2](/versions/8.2.0/basics/hermeticity)
·
[8.1](/versions/8.1.0/basics/hermeticity)

This page covers hermeticity, the benefits of using hermetic builds, and
strategies for identifying non-hermetic behavior in your builds.

## Overview

When given the same input source code and product configuration, a hermetic
build system always returns the same output by isolating the build from changes
to the host system.

In order to isolate the build, hermetic builds are insensitive to libraries and
other software installed on the local or remote host machine. They depend on
specific versions of build tools, such as compilers, and dependencies, such as
libraries. This makes the build process self-contained as it doesn't rely on
services external to the build environment.

The two important aspects of hermeticity are:

* **Isolation**: Hermetic build systems treat tools as source code. They
  download copies of tools and manage their storage and use inside managed file
  trees. This creates isolation between the host machine and local user,
  including installed versions of languages.
* **Source identity**: Hermetic build systems try to ensure the sameness of
  inputs. Code repositories, such as Git, identify sets of code mutations with a
  unique hash code. Hermetic build systems use this hash to identify changes to
  the build's input.

## Benefits

The major benefits of hermetic builds are:

* **Speed**: The output of an action can be cached, and the action need not be
  run again unless inputs change.
* **Parallel execution**: For given input and output, the build system can
  construct a graph of all actions to calculate efficient and parallel
  execution. The build system loads the rules and calculates an action graph
  and hash inputs to look up in the cache.
* **Multiple builds**: You can build multiple hermetic builds on the same
  machine, each build using different tools and versions.
* **Reproducibility**: Hermetic builds are good for troubleshooting because you
  know the exact conditions that produced the build.

## Identifying non-hermeticity

If you are preparing to switch to Bazel, migration is easier if you improve
your existing builds' hermeticity in advance. Some common sources of
non-hermeticity in builds are:

* Arbitrary processing in `.mk` files
* Actions or tooling that create files non-deterministically, usually involving
  build IDs or timestamps
* System binaries that differ across hosts (such as `/usr/bin` binaries, absolute
  paths, system C++ compilers for native C++ rules autoconfiguration)
* Writing to the source tree during the build. This prevents the same source
  tree from being used for another target. The first build writes to the source
  tree, fixing the source tree for target A. Then trying to build target B may
  fail.

## Troubleshooting non-hermetic builds

Starting with local execution, issues that affect local cache hits reveal
non-hermetic actions.

* Ensure null sequential builds: If you run `make` and get a successful build,
  running the build again should not rebuild any targets. If you run each build
  step twice or on different systems, compare a hash of the file contents and
  get results that differ, the build is not reproducible.
* Run steps to
  [debug local cache hits](/remote/cache-remote#troubleshooting-cache-hits)
  from a variety of potential client machines to ensure that you catch any
  cases of client environment leaking into the actions.
* Execute a build within a docker container that contains nothing but the
  checked-out source tree and explicit list of host tools. Build breakages and
  error messages will catch implicit system dependencies.
* Discover and fix hermeticity problems using
  [remote execution rules](/remote/rules#overview).
* Enable strict [sandboxing](/docs/sandboxing)
  at the per-action level, since actions in a build can be stateful and affect
  the build or the output.
* [Workspace rules](/remote/workspace)
  allow developers to add dependencies to external workspaces, but they are
  rich enough to allow arbitrary processing to happen in the process. You can
  get a log of some potentially non-hermetic actions in Bazel workspace rules by
  adding the flag
  `--experimental_workspace_rules_log_file=PATH` to
  your Bazel command.

**Note:** Make your build fully hermetic when mixing remote and local execution,
using Bazel’s “dynamic strategy” functionality. Running Bazel inside the remote
Docker container will enable the build to execute the same in both environments.

## Hermeticity with Bazel

For more information about how other projects have had success using hermetic
builds with Bazel, see these BazelCon talks:

* [Building Real-time Systems with Bazel](https://www.youtube.com/watch?v=t_3bckhV_YI) (SpaceX)
* [Bazel Remote Execution and Remote Caching](https://www.youtube.com/watch?v=_bPyEbAyC0s) (Uber and TwoSigma)
* [Faster Builds With Remote Execution and Caching](https://www.youtube.com/watch?v=MyuJRUwT5LI)
* [Fusing Bazel: Faster Incremental Builds](https://www.youtube.com/watch?v=rQd9Zd1ONOw)
* [Remote Execution vs Local Execution](https://www.youtube.com/watch?v=C8wHmIln--g)
* [Improving the Usability of Remote Caching](https://www.youtube.com/watch?v=u5m7V3ZRHLA) (IBM)
* [Building Self Driving Cars with Bazel](https://www.youtube.com/watch?v=Gh4SJuYUoQI&list=PLxNYxgaZ8Rsf-7g43Z8LyXct9ax6egdSj&index=4&t=0s) (BMW)
* [Building Self Driving Cars with Bazel + Q&A](https://www.youtube.com/watch?v=fjfFe98LTm8&list=PLxNYxgaZ8Rsf-7g43Z8LyXct9ax6egdSj&index=29) (GM Cruise)

Was this helpful?

Send feedback

Except as otherwise noted, the content of this page is licensed under the [Creative Commons Attribution 4.0 License](https://creativecommons.org/licenses/by/4.0/), and code samples are licensed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0). For details, see the [Google Developers Site Policies](https://developers.google.com/site-policies). Java is a registered trademark of Oracle and/or its affiliates.

Last updated 2026-05-07 UTC.

Need to tell us more?

[[["Easy to understand","easyToUnderstand","thumb-up"],["Solved my problem","solvedMyProblem","thumb-up"],["Other","otherUp","thumb-up"]],[["Missing the information I need","missingTheInformationINeed","thumb-down"],["Too complicated / too many steps","tooComplicatedTooManySteps","thumb-down"],["Out of date","outOfDate","thumb-down"],["Samples / code issue","samplesCodeIssue","thumb-down"],["Other","otherDown","thumb-down"]],["Last updated 2026-05-07 UTC."],[],[]]

* ### About

  + [Who's using Bazel](/community/users)
  + [Contribute](/contribute/)
  + [Governance model](/contribute/contribution-policy)
  + [Release model](/release)
  + [Brand guidelines](/brand)
* ### Stay connected

  + [Blog](//blog.bazel.build)
  + [GitHub](//github.com/bazelbuild/bazel)
  + [Twitter](//twitter.com/bazelbuild)
  + [YouTube](//youtube.com/user/googleOSPO)
* ### Support

  + [Support](/help)
  + [Issue tracker](//github.com/bazelbuild/bazel/issues)
  + [Slack](//slack.bazel.build)
  + [Stack Overflow](//stackoverflow.com/questions/tagged/bazel)

* [Terms](//policies.google.com/terms)
* [Privacy](//policies.google.com/privacy)
* [Manage cookies](#)

* [English](https://bazel.build/basics/hermeticity)
* [Español – América Latina](https://bazel.build/basics/hermeticity?hl=es-419)
* [Indonesia](https://bazel.build/basics/hermeticity?hl=id)
* [Português – Brasil](https://bazel.build/basics/hermeticity?hl=pt-br)
* [Tiếng Việt](https://bazel.build/basics/hermeticity?hl=vi)
* [Türkçe](https://bazel.build/basics/hermeticity?hl=tr)
* [עברית](https://bazel.build/basics/hermeticity?hl=he)
* [العربيّة](https://bazel.build/basics/hermeticity?hl=ar)
* [فارسی](https://bazel.build/basics/hermeticity?hl=fa)
* [हिंदी](https://bazel.build/basics/hermeticity?hl=hi)
* [বাংলা](https://bazel.build/basics/hermeticity?hl=bn)
* [ภาษาไทย](https://bazel.build/basics/hermeticity?hl=th)
* [中文 – 简体](https://bazel.build/basics/hermeticity?hl=zh-cn)
* [中文 – 繁體](https://bazel.build/basics/hermeticity?hl=zh-tw)
* [日本語](https://bazel.build/basics/hermeticity?hl=ja)
* [한국어](https://bazel.build/basics/hermeticity?hl=ko)
