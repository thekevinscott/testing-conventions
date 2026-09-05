import { writeFile } from 'node:fs/promises';

import { parseArgs } from './parse-args.js';
import { runStryker } from './run-stryker.js';

/**
 * The TypeScript mutation adapter: run Stryker over `argv`'s scope and emit the normalized results
 * as JSON, to `--out <path>` when given and to stdout otherwise.
 */
export async function mutationCLI(argv: string[]): Promise<void> {
  const { mutate, out, testFiles } = parseArgs(argv);
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
