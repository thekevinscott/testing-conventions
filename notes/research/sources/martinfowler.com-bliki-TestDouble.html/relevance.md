# Relevance — Test Double

**Verdict:** relevant (broad-inclusion) — the canonical definitions of the five test doubles, supplying the precise terminology a strategy needs when specifying mocking choices for LLM-authored code; purely definitional, no argument or evidence.

## Salient sections

- Umbrella definition (opening): "Test Double is a generic term for any case where you replace a production object for testing purposes." → fixes the term the goal uses under "test-doubles/mocking"; definitional.
- The five kinds (definition list): Dummy ("passed around but never actually used"), Fake ("working implementations, but... take some shortcut... an InMemoryTestDatabase"), Stub ("provide canned answers to calls made during the test"), Spy ("stubs that also record some information based on how they were called"), Mock ("pre-programmed with expectations which form a specification of the calls they are expected to receive") → gives the controlled vocabulary for distinguishing mocking strategies when testing LLM-generated code; attributed to Gerard Meszaros, popularized by Fowler.

## Evidence weight

Type = definitional glossary entry by Martin Fowler (Meszaros's taxonomy, no method or data); non-empirical terminology context, not primary evidence.
