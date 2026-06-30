import { isAbsolute, relative, sep } from 'node:path';
import type { MutantResult } from '@stryker-mutator/api/core';

import { normalizeStatus, type NormalizedStatus } from './normalize-status.js';

/**
 * One mutant in the normalized result set — the engine-agnostic shape the Rust core reads
 * (`mutation::NormalizedMutant`). The adapter emits an array of these so the core never sees
 * a Stryker-specific report.
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
 * Map one Stryker `MutantResult` to a {@link NormalizedMutant}, or `null` when its status is
 * one the gate ignores (see {@link normalizeStatus}).
 *
 * Stryker reports `fileName` as an absolute path, but the gate keys on project-relative,
 * `/`-separated paths (so exemptions and the `--base` diff match), so it's made relative to
 * `projectRoot` and separators are normalized.
 */
export function toNormalized(result: MutantResult, projectRoot: string): NormalizedMutant | null {
  const status = normalizeStatus(result.status);
  if (status === null) {
    return null;
  }
  const file = (isAbsolute(result.fileName) ? relative(projectRoot, result.fileName) : result.fileName)
    .split(sep)
    .join('/');
  return {
    file,
    line: result.location.start.line,
    status,
    mutator: result.mutatorName,
    ...(result.replacement === undefined ? {} : { replacement: result.replacement }),
  };
}
