#!/usr/bin/env node
import { fileURLToPath } from 'node:url';
import { main } from 'bin-shim';

// The TypeScript `unit mutation` arm runs Stryker through the bundled Node adapter (#246):
// the rust binary spawns `node` on it, but a Rust binary can't reliably locate a JS file in
// the npm tree. So the Node launcher — which knows its own `dist/` — hands the binary the
// adapter's path via this env var (the binary reads it; absent ⇒ a clear "run via the npm
// distribution" error). `??=` leaves an explicit value in place, so tests / unusual layouts
// can point at a different build.
process.env.TESTING_CONVENTIONS_TS_MUTATION_ADAPTER ??= fileURLToPath(
  new URL('./mutation-cli.js', import.meta.url),
);

main({
  scope: 'testing-conventions',
  binaryName: 'testing-conventions',
  from: import.meta.url,
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
