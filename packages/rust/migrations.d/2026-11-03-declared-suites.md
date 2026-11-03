### Declared suite scanner arguments

#### Summary

The public Rust scanner helpers take declared suite names explicitly so integration-lint can scan additional Python and TypeScript suite directories.

#### Required changes

Pass a suite list as the second argument to `lint::find_suite_violations` and `ts::find_suite_violations`. Use `&[]` to scan the standard suites only. Code constructing `Config` directly supplies `integration_lint: None` or an `IntegrationLintConfig` value.

#### Deprecations removed

_None._

#### Behavior changes without code changes

`[integration-lint].suites` adds package-root suite directories to the check. Undeclared test files continue to receive `unknown-tier`.

#### Verification

Run `cargo test --test integration_lint --test ts_integration_lint` in `packages/rust`; both suites pass.
