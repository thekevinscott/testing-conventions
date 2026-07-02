import type { MutantStatus } from '@stryker-mutator/api/core';

/** The normalized status vocabulary shared with the Rust core (`snake_case` on the wire). */
export type NormalizedStatus =
  | 'survived'
  | 'killed'
  | 'no_coverage'
  | 'timeout'
  | 'compile_error'
  | 'runtime_error';

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
