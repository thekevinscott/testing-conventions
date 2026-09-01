import { mutationCLI } from './mutation-cli.js';

// The executable the rust binary spawns for the TS mutation arm: it runs `mutationCLI` over the
// process arguments and maps a failed run onto a non-zero exit code. Separate from
// `mutation-cli.ts`, which stays a pure importable function with no process side effects.
mutationCLI(process.argv.slice(2)).catch((err: Error) => {
  process.stderr.write(`${err.message}\n`);
  process.exitCode = 1;
});
