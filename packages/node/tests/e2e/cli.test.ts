import { execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// E2E: drive the real, source-built CLI against a real temp directory. `e2e attest` records
// the local run and the dogfood `e2e verify` gate checks the committed attestation.
// The binary is the dogfood job's source build; TESTING_CONVENTIONS_BIN overrides its path.
const here = fileURLToPath(new URL('.', import.meta.url));
const bin =
  process.env.TESTING_CONVENTIONS_BIN ??
  resolve(here, '../../../rust/target/release/testing-conventions');

// execFileSync throws on a non-zero exit, carrying the code as `status`.
function colocatedTestExit(dir: string): number {
  try {
    execFileSync(bin, ['unit', 'colocated-test', '--language', 'typescript', dir], {
      stdio: 'pipe',
    });
    return 0;
  } catch (err) {
    if (err && typeof err === 'object' && 'status' in err && typeof err.status === 'number') {
      return err.status;
    }
    return 1;
  }
}

describe('testing-conventions CLI (e2e)', () => {
  it('passes a colocated suite and flags an orphan — real binary, no mocks', () => {
    const clean = mkdtempSync(join(tmpdir(), 'tc-e2e-clean-'));
    writeFileSync(join(clean, 'widget.ts'), 'export const widget = 1;\n');
    writeFileSync(join(clean, 'widget.test.ts'), 'export const widgetTest = 1;\n');
    expect(colocatedTestExit(clean)).toBe(0);

    const orphaned = mkdtempSync(join(tmpdir(), 'tc-e2e-orphan-'));
    writeFileSync(join(orphaned, 'widget.ts'), 'export const widget = 1;\n');
    expect(colocatedTestExit(orphaned)).toBe(1);
  });
});
