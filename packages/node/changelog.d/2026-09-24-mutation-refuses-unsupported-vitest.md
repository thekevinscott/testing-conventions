**Fixed** The TypeScript mutation adapter refuses to run on vitest 5 and above instead of
reporting every mutant as survived. Stryker's vitest runner completes zero tests per mutant from
vitest 5 on — the dry run and per-test coverage attribution both still succeed, so each mutant
scored as `survived` even when the suite killed it. Verified against
`@stryker-mutator/vitest-runner` 9.6.1 and 10.0.0; vitest 4 and below are unaffected. The adapter
now fails with the version it found and the pin that works. **BREAKING** for a vitest 5 project
whose `mutation` exemptions lifted those false survivors — see
`../migrations.d/2026-09-24-mutation-refuses-unsupported-vitest.md`.
