**Fixed** Python `unit mutation` no longer runs `tests/integration/` or `tests/e2e/` as part of
the per-mutant suite when the scanned module path is itself the package root. cosmic-ray's
`test-command` now carries `--ignore=tests`, the same exclusion `unit coverage`'s pytest
invocation applies, so a surviving mutant in the integration or e2e tier can no longer be killed
(or missed) by a suite that was never meant to run under the unit baseline.
