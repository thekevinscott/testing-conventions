// Builds the publishable JS shim. The putitoutthere workflow invokes it on every npm row; on a
// per-triple row TARGET names a rust triple and the engine stages the binary, so it exits early.

import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { buildShim } from './build-shim.js';

const here = dirname(fileURLToPath(import.meta.url));
const nodePkg = resolve(here, '..');

process.exit(buildShim(spawnSync, process.env.TARGET ?? '', nodePkg));
