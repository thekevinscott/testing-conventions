import { writeFile } from 'node:fs/promises';

import { parseArgs } from './parse-args.js';
import { readVitestVersion } from './read-vitest-version.js';
import { runStryker } from './run-stryker.js';
import { unsupportedVitestReason } from './unsupported-vitest-reason.js';

/**
 * The TypeScript mutation adapter: run Stryker over `argv`'s scope and emit the normalized results
 * as JSON, to `--out <path>` when given and to stdout otherwise. Refuses outright on a vitest the
 * runner silently mis-scores, so the gate never reports a survivor it did not actually judge.
 */
export async function mutationCLI(argv: string[]): Promise<void> {
  const { mutate, out, testFiles } = parseArgs(argv);
  const unsupported = unsupportedVitestReason(readVitestVersion(process.cwd()));
  if (unsupported !== null) {
    throw new Error(unsupported);
  }
  const results = await runStryker({
    ...(mutate === undefined ? {} : { mutate }),
    ...(testFiles === undefined ? {} : { testFiles }),
  });
  const json = `${JSON.stringify(results)}\n`;
  if (out === undefined) {
    process.stdout.write(json);
  } else {
    await writeFile(out, json);
  }
}
