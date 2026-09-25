import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { bootstrapManifest } from './bootstrap-manifest.js';
import { bootstrapNames } from './bootstrap-names.js';

/** The I/O `publishBootstrapStubs` performs, typed to the `node:fs` and `node:child_process`
 * signatures themselves, so the composition root passes those through unwrapped. */
export interface BootstrapIo {
  makeTempDir: (prefix: string) => string;
  writeFile: (path: string, contents: string) => void;
  run: (
    command: string,
    args: string[],
    options: { cwd: string; stdio: 'inherit' },
  ) => { status: number | null };
  removeDir: (path: string, options: { recursive: true; force: true }) => void;
}

/** Publishes a bootstrap stub for every name in a comma-separated `packages` input, each from a
 * temporary directory holding only its manifest. Throws on the first name npm rejects. */
export function publishBootstrapStubs(io: BootstrapIo, packages: string): void {
  for (const name of bootstrapNames(packages)) {
    const dir = io.makeTempDir(join(tmpdir(), 'bootstrap-npm-'));
    try {
      const manifest = JSON.stringify(bootstrapManifest(name), null, 2);
      io.writeFile(join(dir, 'package.json'), `${manifest}\n`);
      // Dropping `--tag bootstrap` fails the publish: npm refuses a prerelease as `latest`, which
      // the first real release — not a prerelease — takes back.
      const publish = ['publish', '--access', 'public', '--tag', 'bootstrap'];
      const { status } = io.run('npm', publish, { cwd: dir, stdio: 'inherit' });
      if (status !== 0) {
        throw new Error(`publishing the ${name} stub failed: npm exited ${String(status)}`);
      }
    } finally {
      io.removeDir(dir, { recursive: true, force: true });
    }
  }
}
