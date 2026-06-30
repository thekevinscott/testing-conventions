import { writeFileSync } from 'node:fs';

import { parseArgs } from './parse-args.js';
import { runStryker } from './run-stryker.js';

// CLI entry for the TypeScript mutation adapter (#239 / #246). The rust binary spawns `node`
// on the built form of this file for the TS arm — its path is injected by `bin.ts` via an env
// var, so the binary never hunts the filesystem for it. It runs Stryker through the Node API
// and emits the normalized results as JSON. `--out <path>` writes them to a file (the rule
// passes a temp file, so Stryker's own stdout logging can't corrupt them); without it, the
// JSON goes to stdout. `--mutate <a,b,...>` scopes the run to those Stryker mutate patterns.
const { mutate, out } = parseArgs(process.argv.slice(2));

runStryker(mutate === undefined ? {} : { mutate })
  .then((results) => {
    const json = `${JSON.stringify(results)}\n`;
    if (out === undefined) {
      process.stdout.write(json);
    } else {
      writeFileSync(out, json);
    }
  })
  .catch((err: Error) => {
    process.stderr.write(`${err.message}\n`);
    process.exitCode = 1;
  });
