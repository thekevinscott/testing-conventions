import { mutationCLI } from './index.js';

// The executable the rust binary spawns for the TS mutation arm (#246): `dist/mutation/main.js`
// runs `mutationCLI` over the process arguments and maps a failed run onto a non-zero exit code.
// Kept separate from `index.ts` so the orchestration stays a pure, importable function with no
// process side effects (and each stays fully covered).
mutationCLI(process.argv.slice(2)).catch((err: Error) => {
  process.stderr.write(`${err.message}\n`);
  process.exitCode = 1;
});
