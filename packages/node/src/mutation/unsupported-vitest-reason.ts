// Stryker's vitest runner completes zero tests per mutant from this major on: the dry run and
// per-test coverage both still succeed, so every mutant scores as `survived` rather than erroring.
// Verified against @stryker-mutator/vitest-runner 9.6.1 and 10.0.0; vitest 4 is unaffected.
const FIRST_UNSUPPORTED_MAJOR = 5;

/**
 * Why the mutation gate refuses to run against `version`, or `null` when it will run. An absent or
 * unparseable version yields `null` — the runner reports its own missing-vitest failure better than
 * a guess here would.
 */
export function unsupportedVitestReason(version: string | null): string | null {
  if (version === null) {
    return null;
  }
  const major = Number.parseInt(version, 10);
  if (!Number.isInteger(major) || major < FIRST_UNSUPPORTED_MAJOR) {
    return null;
  }
  return (
    `the mutation gate cannot judge mutants on vitest ${version}: Stryker's vitest runner runs no ` +
    `tests per mutant on vitest ${FIRST_UNSUPPORTED_MAJOR} and above, so every mutant reports as ` +
    `survived even when the suite kills it. Pin \`vitest\` and \`@vitest/coverage-v8\` to \`^4\` ` +
    `to run this gate, or leave \`mutation\` out of the workflow's \`gates\` to skip it. ` +
    `See https://github.com/thekevinscott/testing-conventions/issues/700.`
  );
}
