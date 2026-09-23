import type { SpawnSyncOptions } from 'node:child_process';

import { shouldBuildShim } from './build-decision.js';

/** The slice of `spawnSync`'s signature `buildShim` calls — matches `spawnSync` itself, so the
 * composition root passes it through unwrapped rather than adapting it inline. */
export type Spawn = (
  command: string,
  args: string[],
  options: SpawnSyncOptions,
) => { status: number | null };

const TSC_OPTS = { stdio: 'inherit', shell: true } as const;

/**
 * Builds the JS shim for a row `shouldBuildShim` says builds one, or logs and returns `0` for a
 * per-triple row the engine already staged. `spawn` is injected so both branches, and a failing
 * `tsc` invocation's exit-code propagation, are testable against a fake.
 */
export function buildShim(spawn: Spawn, target: string, packageRoot: string): number {
  if (!shouldBuildShim(target)) {
    console.log(`nothing to build for ${target}: the engine stages the binary`);
    return 0;
  }

  const opts: SpawnSyncOptions = { ...TSC_OPTS, cwd: packageRoot };
  const clean = spawn('npx', ['--no-install', 'tsc', '-b', '--clean', 'tsconfig.json'], opts);
  if (clean.status !== 0) {
    console.error(`failed: npx tsc -b --clean tsconfig.json (exit ${clean.status})`);
    return clean.status ?? 1;
  }

  const build = spawn('npx', ['--no-install', 'tsc', '-p', 'tsconfig.json'], opts);
  if (build.status !== 0) {
    console.error(`failed: npx tsc -p tsconfig.json (exit ${build.status})`);
    return build.status ?? 1;
  }

  return 0;
}
