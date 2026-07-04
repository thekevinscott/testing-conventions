---
layout: home

hero:
  name: 'testing-conventions'
  text: 'Enforce testing conventions in CI'
  tagline: 'Opinionated testing standards enforcing structure, measurement, and isolation. For Python, TypeScript, and Rust.'
  actions:
    - theme: brand
      text: Getting Started
      link: /getting-started
    - theme: alt
      text: Understand the checks
      link: /explanation/

features:
  - title: Getting Started
    details: The five-minute drop-in — add one workflow file, watch a check go red on a pull request, make it green.
    link: /getting-started
    linkText: Start the tutorial
  - title: Adopt on a monorepo
    details: One workflow call per package, each scoped to its own source directory, language, and config.
    link: /monorepo
    linkText: Adopt per package
  - title: The checks
    details: What each check enforces and why — the three kinds of test, the unit ladder, and what makes it agent-resistant.
    link: /explanation/
    linkText: Understand the model
  - title: Configure the rules
    details: The two responses to a red check — fix the code, or record a reasoned exemption in one auditable file.
    link: /guide/configure
    linkText: Tune a floor, exempt a file
---

<!--@include: ../README.md#rules-->
