#!/usr/bin/env node
import { fileURLToPath } from 'node:url';
import { main } from 'bin-shim';

// The TypeScript `unit mutation` arm runs Stryker through the bundled Node adapter (#246): the
// rust binary spawns `node` on it, but a Rust binary can't reliably locate a JS file in the npm
// tree. The Node launcher — which knows its own `dist/` — hands the binary the adapter's path as
// an explicit `--ts-mutation-adapter` CLI argument, appended only to a `unit mutation` invocation
// (the only command that reads it). The binary errors clearly if the arm runs without it.
const args = process.argv.slice(2);
const isUnitMutation = args[0] === 'unit' && args[1] === 'mutation';
const adapter = fileURLToPath(new URL('../mutation/main.js', import.meta.url));
const argv = isUnitMutation ? [...args, '--ts-mutation-adapter', adapter] : args;

main({
  scope: 'testing-conventions',
  binaryName: 'testing-conventions',
  from: import.meta.url,
  argv,
  platformPackage: '@{scope}/{triple}',
  triples: {
    'linux-x64': 'x86_64-unknown-linux-gnu',
    'linux-arm64': 'aarch64-unknown-linux-gnu',
    'darwin-x64': 'x86_64-apple-darwin',
    'darwin-arm64': 'aarch64-apple-darwin',
    'win32-x64': 'x86_64-pc-windows-msvc',
  },
})
  .then((code) => process.exit(code))
  .catch((err: Error) => {
    process.stderr.write(`${err.message}\n`);
    process.exit(1);
  });
