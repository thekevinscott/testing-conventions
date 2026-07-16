import { parseArgs as parseNodeArgs } from 'node:util';

/** The CLI arguments the TypeScript mutation adapter understands. */
export interface AdapterArgs {
  /** Stryker `mutate` patterns (`--mutate a,b,...`), or `undefined` for the default set. */
  mutate?: string[];
  /** File to write the normalized-results JSON to (`--out <path>`); stdout when absent. */
  out?: string;
  /** Directory vitest discovers tests in (`--vitest-dir <path>`), relative to the project root. */
  vitestDir?: string;
}

/**
 * Parse the adapter's CLI arguments with Node's built-in `util.parseArgs`: `--out <path>` (where
 * to write the normalized JSON — the rule passes a temp file so Stryker's own stdout logging can't
 * corrupt the results), `--mutate <a,b,...>` (Stryker mutate patterns, comma-split into a list),
 * and `--vitest-dir <path>` (the directory vitest discovers tests in — the scan path within the
 * package, so the colocated unit suite alone judges the mutants). All are optional; the rust
 * binary supplies the argv, so it stays a fixed, controlled shape.
 */
export function parseArgs(argv: string[]): AdapterArgs {
  const { values } = parseNodeArgs({
    args: argv,
    options: {
      mutate: { type: 'string' },
      out: { type: 'string' },
      'vitest-dir': { type: 'string' },
    },
    allowPositionals: true,
  });

  const result: AdapterArgs = {};
  if (values.mutate !== undefined) {
    result.mutate = values.mutate.split(',');
  }
  if (values.out !== undefined) {
    result.out = values.out;
  }
  if (values['vitest-dir'] !== undefined) {
    result.vitestDir = values['vitest-dir'];
  }
  return result;
}
