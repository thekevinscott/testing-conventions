// Builds the publishable JS shim. The putitoutthere workflow invokes it on every npm row; on a
// per-triple row TARGET names a rust triple and the engine stages the binary, so it exits early.

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

// `--no-install` uses the locally-installed tsc whichever package manager populated
// node_modules (`npm` at release-time, `pnpm` at PR-time).
run('npx', ['--no-install', 'tsc', '-b', '--clean', 'tsconfig.json'], { cwd: nodePkg });
run('npx', ['--no-install', 'tsc', '-p', 'tsconfig.json'], { cwd: nodePkg });

function run(cmd: string, args: string[], opts: SpawnSyncOptions = {}): void {
  // `shell: true` so Windows resolves the `.cmd` shims (npx.cmd, tsc.cmd) these names need.
  const res = spawnSync(cmd, args, { stdio: 'inherit', shell: true, ...opts });
  if (res.status !== 0) {
    console.error(`failed: ${cmd} ${args.join(' ')} (exit ${res.status})`);
    process.exit(res.status ?? 1);
  }
}
