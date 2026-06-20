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
      text: Guides
      link: /guide/

---

<!--@include: ../README.md#rules-->

## Beyond the coverage floor

Coverage proves your tests *ran* the code; it can't prove they *checked* it.
[Mutation testing](/guide/mutation) — the planned `unit mutation` rule — closes that
gap, and is the verification signal an agent can't fake.
