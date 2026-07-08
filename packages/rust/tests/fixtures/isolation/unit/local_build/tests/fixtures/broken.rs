// An intentionally-unparsable `.rs` under the crate's own `tests/fixtures/` (a crate
// may ship broken sources as test fixtures). The unit-isolation walk must skip `tests/`
// entirely, so this file is never parsed and never aborts the rule.
this is not valid rust !!! fn ( { <
