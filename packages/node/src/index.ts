import { writeFile } from 'node:fs/promises';

import { parseArgs } from './mutation/parse-args.js';
import { runStryker } from './mutation/run-stryker.js';

/**
 * The TypeScript mutation adapter (#239 / #246). The rust binary spawns the built entry on the
 * TS arm — its path comes from the launcher's `--ts-mutation-adapter` argument, so the binary
 * never hunts the filesystem for it. It runs Stryker through the Node API and emits the
 * normalized results as JSON. `--out <path>` writes them to a file (the rule passes a temp file,
 * so Stryker's own stdout logging can't corrupt them); without it, the JSON goes to stdout.
 * `--mutate <a,b,...>` scopes the run to those Stryker mutate patterns. Rejects on a failed run;
 * the executable shim (`mutation-cli.ts`) maps that onto a non-zero exit code.
 */
export async function mutationCLI(argv: string[]): Promise<void> {
  const { mutate, out } = parseArgs(argv);
  const results = await runStryker(mutate === undefined ? {} : { mutate });
  const json = `${JSON.stringify(results)}\n`;
  if (out === undefined) {
    process.stdout.write(json);
  } else {
    await writeFile(out, json);
  }
}
