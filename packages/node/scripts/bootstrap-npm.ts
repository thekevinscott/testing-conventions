// Publishes the bootstrap stubs `.github/workflows/bootstrap-npm.yml` dispatches, naming them in
// PACKAGES. See docs/internals/repo.md, "Bootstrapping a new npm package name".

import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';

import { publishBootstrapStubs } from './publish-bootstrap-stubs.js';

publishBootstrapStubs(
  {
    makeTempDir: mkdtempSync,
    writeFile: writeFileSync,
    run: spawnSync,
    removeDir: rmSync,
  },
  process.env.PACKAGES ?? '',
);
