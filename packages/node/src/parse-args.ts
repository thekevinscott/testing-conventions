/** The CLI arguments the TypeScript mutation adapter understands. */
export interface AdapterArgs {
  /** Stryker `mutate` patterns (`--mutate a,b,...`), or `undefined` for the default set. */
  mutate?: string[];
  /** File to write the normalized-results JSON to (`--out <path>`); stdout when absent. */
  out?: string;
}

/**
 * Parse the adapter's CLI arguments: `--mutate <a,b,...>` (comma-separated Stryker mutate
 * patterns) and `--out <path>` (where to write the normalized JSON; the rule passes a temp
 * file so Stryker's own stdout logging never corrupts the results). A flag with no following
 * value is treated as absent.
 */
export function parseArgs(argv: string[]): AdapterArgs {
  const args: AdapterArgs = {};

  const mutateIdx = argv.indexOf('--mutate');
  const mutateValue = mutateIdx === -1 ? undefined : argv[mutateIdx + 1];
  if (mutateValue !== undefined) {
    args.mutate = mutateValue.split(',');
  }

  const outIdx = argv.indexOf('--out');
  const outValue = outIdx === -1 ? undefined : argv[outIdx + 1];
  if (outValue !== undefined) {
    args.out = outValue;
  }

  return args;
}
