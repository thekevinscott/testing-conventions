#!/usr/bin/env node
// Latency harness (Node side): FFI vs WASM vs subprocess.
// Run from spike/ after `napi build --platform --release` and the wasm build.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const SPIKE = dirname(dirname(fileURLToPath(import.meta.url)));
const TREE = join(SPIKE, 'bench', 'tree');
const NATIVE = join(SPIKE, 'target', 'release', 'agent-context');
const GLOBS = ['**/*.md'];
const WARM_N = 1000;
const COLD_N = 20;

const medianMs = (fn, n) => {
  const times = [];
  for (let i = 0; i < n; i++) {
    const t0 = process.hrtime.bigint();
    fn();
    times.push(Number(process.hrtime.bigint() - t0) / 1e6);
  }
  return times.sort((a, b) => a - b)[Math.floor(n / 2)];
};

const sha = (text) => createHash('sha256').update(text).digest('hex').slice(0, 12);

const results = {};

// --- subprocess-over-native-binary baseline ---
const sub = () => execFileSync(NATIVE, ['print', ...GLOBS, '--cwd', TREE], { encoding: 'utf8' });
results['subprocess warm=cold'] = medianMs(sub, COLD_N);
const subText = sub();

// --- FFI (napi-rs) ---
const ffi = await import(join(SPIKE, 'ffi', 'node', 'index.js')).then(m => m.default ?? m);
results['ffi warm'] = medianMs(() => ffi.print(GLOBS, TREE), WARM_N);
const ffiR = ffi.print(GLOBS, TREE);
const ffiCold = `const ffi = require(${JSON.stringify(join(SPIKE, 'ffi', 'node', 'index.js'))}); ffi.print(${JSON.stringify(GLOBS)}, ${JSON.stringify(TREE)})`;
results['ffi cold (fresh node)'] = medianMs(
  () => execFileSync(process.execPath, ['-e', ffiCold]), COLD_N);

// --- WASM (jco) ---
// NOTE: the jco-transpiled module instantiates once at import time (module
// singleton) — there is no supported in-process re-instantiation, so cold
// for jco is a fresh node process.
const { sandbox, print } = await import(join(SPIKE, 'wasm', 'node', 'sdk.mjs'));
const guest = sandbox(TREE);
results['wasm warm'] = medianMs(() => print(GLOBS, guest), WARM_N);
const wasmR = print(GLOBS, guest);
const wasmCold = `import(${JSON.stringify(join(SPIKE, 'wasm', 'node', 'sdk.mjs'))}).then(s => s.print(${JSON.stringify(GLOBS)}, s.sandbox(${JSON.stringify(TREE)})))`;
results['wasm cold (fresh node)'] = medianMs(
  () => execFileSync(process.execPath, ['-e', wasmCold]), COLD_N);

// --- runtime-startup floor for cold context ---
results['node noop (fresh node)'] = medianMs(
  () => execFileSync(process.execPath, ['-e', '0']), COLD_N);

console.log(JSON.stringify(Object.fromEntries(
  Object.entries(results).map(([k, v]) => [k, Math.round(v * 1000) / 1000])), null, 2));
console.log('parity sha256/12:', {
  subprocess: sha(subText),
  ffi: sha(ffiR.text),
  wasm: sha(wasmR.text),
});
