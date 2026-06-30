import { Stryker } from '@stryker-mutator/core';
import type { MutantResult, MutantStatus, PartialStrykerOptions } from '@stryker-mutator/api/core';

// The TypeScript mutation engine adapter (#239 / #246). It drives Stryker through its
// **Node API** — `new Stryker(options).runMutationTest()` — rather than spawning a CLI and
// parsing a report file, and maps the structured `MutantResult[]` it returns onto the
// engine-agnostic normalized schema the Rust core gates on (`mutation::NormalizedMutant`).
// Because this runs inside the npm package, `@stryker-mutator/core` resolves natively — no
// cross-ecosystem hunting, and the consumer installs only their own test runner (vitest).

/** The normalized status vocabulary shared with the Rust core (`snake_case` on the wire). */
export type NormalizedStatus =
  | 'survived'
  | 'killed'
  | 'no_coverage'
  | 'timeout'
  | 'compile_error'
  | 'runtime_error';

/**
 * One mutant in the normalized result set — the engine-agnostic shape the Rust core reads
 * (`mutation::NormalizedMutant`). The adapter emits an array of these so the core never
 * sees a Stryker-specific report.
 */
export interface NormalizedMutant {
  /** Project-relative, `/`-separated path of the mutated file. */
  file: string;
  /** 1-based line the mutant starts on. */
  line: number;
  /** The outcome, normalized across engines. */
  status: NormalizedStatus;
  /** Stryker's mutator name (e.g. `ConditionalExpression`). */
  mutator: string;
  /** The replacement text, when Stryker reports one. */
  replacement?: string;
}

/**
 * Map a Stryker `MutantStatus` onto the normalized vocabulary, or `null` for the two
 * non-outcomes the gate doesn't model: `Ignored` (excluded by config) and `Pending` (never
 * ran). Every other Stryker status has a 1:1 normalized counterpart.
 */
export function normalizeStatus(status: MutantStatus): NormalizedStatus | null {
  switch (status) {
    case 'Survived':
      return 'survived';
    case 'Killed':
      return 'killed';
    case 'NoCoverage':
      return 'no_coverage';
    case 'Timeout':
      return 'timeout';
    case 'CompileError':
      return 'compile_error';
    case 'RuntimeError':
      return 'runtime_error';
    case 'Ignored':
    case 'Pending':
      return null;
  }
}

/**
 * Map one Stryker `MutantResult` to a {@link NormalizedMutant}, or `null` when its status is
 * one the gate ignores (see {@link normalizeStatus}).
 */
export function toNormalized(result: MutantResult): NormalizedMutant | null {
  const status = normalizeStatus(result.status);
  if (status === null) {
    return null;
  }
  return {
    file: result.fileName,
    line: result.location.start.line,
    status,
    mutator: result.mutatorName,
    ...(result.replacement === undefined ? {} : { replacement: result.replacement }),
  };
}

/** Options for {@link runStryker}. */
export interface RunStrykerOptions {
  /**
   * Stryker `mutate` patterns to scope the run to (e.g. `<file>:<start>-<end>` ranges for a
   * diff-scoped gate). Omitted ⇒ Stryker's configured/default `mutate` set.
   */
  mutate?: string[];
}

/**
 * Run Stryker over the project in the current working directory via its Node API and return
 * the normalized results (#239). The **bundled** vitest runner
 * (`@stryker-mutator/vitest-runner`) is selected explicitly, so the unit-scoped runner is
 * always used rather than Stryker's default `npm test` command runner (#240); results are
 * read from `runMutationTest()` directly, so there is no CLI spawn and no report file.
 */
export async function runStryker(options: RunStrykerOptions = {}): Promise<NormalizedMutant[]> {
  const cliOptions: PartialStrykerOptions = {
    testRunner: 'vitest',
    // Results come from runMutationTest()'s return value, so no file/stdout reporter is needed.
    reporters: [],
    ...(options.mutate ? { mutate: options.mutate } : {}),
  };
  const results = await new Stryker(cliOptions).runMutationTest();
  return results
    .map(toNormalized)
    .filter((mutant): mutant is NormalizedMutant => mutant !== null);
}
