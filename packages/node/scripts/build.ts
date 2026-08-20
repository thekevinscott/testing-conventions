// Builds the publishable JS shim (tsc). Invoked by the putitoutthere reusable
// workflow on every npm row, including per-triple ones — there TARGET names a
// rust triple and the engine's bundled-cli build owns the cross-compile and
// staging (version-stamped, flat at the platform-package root), so this script
// exits without building.
//
// Run via tsx (see the `build` script in package.json).

import { spawnSync, type SpawnSyncOptions } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const nodePkg = resolve(here, '..');

const target = process.env.TARGET ?? '';

if (target !== '' && target !== 'main' && target !== 'noarch') {
  console.log(`nothing to build for ${target}: the engine stages the binary`);
  process.exit(0);
}

// Use the locally-installed tsc, regardless of which package manager
// (`npm` at release-time, `pnpm` at PR-time) populated node_modules.
run('npx', ['--no-install', 'tsc', '-b', '--clean', 'tsconfig.json'], { cwd: nodePkg });
run('npx', ['--no-install', 'tsc', '-p', 'tsconfig.json'], { cwd: nodePkg });

function run(cmd: string, args: string[], opts: SpawnSyncOptions = {}): void {
  // shell: true so Windows resolves `.cmd` shims (npx.cmd, tsc.cmd, etc.)
  // without each call hard-coding extensions. Args are static — no injection.
  const res = spawnSync(cmd, args, { stdio: 'inherit', shell: true, ...opts });
  if (res.status !== 0) {
    console.error(`failed: ${cmd} ${args.join(' ')} (exit ${res.status})`);
    process.exit(res.status ?? 1);
  }
}
