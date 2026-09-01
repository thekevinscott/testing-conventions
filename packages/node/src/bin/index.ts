#!/usr/bin/env node
import { fileURLToPath } from 'node:url';
import { main } from 'bin-shim';

// The rust binary cannot locate the bundled mutation adapter in the npm tree, so the launcher —
// which knows its own `dist/` — passes the path as `--ts-mutation-adapter` on the one command
// that reads it. The binary errors if the arm runs without it.
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
  // putitoutthere's bundled-cli build stages the binary at the platform-package root.
  binaryDir: '',
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
