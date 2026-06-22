import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// E2E: a consumer imports the base config by the package's public specifier
// (`import { vitestConfig } from 'testing-conventions'`). Drive that bare
// specifier through a real node process so the `exports` map + built `dist`
// resolve end to end — no mocks. CI never runs this (#71 attestation); run it
// with `npm run test:e2e`, which builds `dist` first.
const here = fileURLToPath(new URL('.', import.meta.url));
const packageRoot = resolve(here, '../..');

const probe = [
  "import { vitestConfig } from 'testing-conventions';",
  'const c = vitestConfig.test.coverage;',
  'process.stdout.write(JSON.stringify({',
  '  provider: c.provider,',
  '  include: c.include,',
  '  exclude: c.exclude,',
  '  thresholds: c.thresholds,',
  '  testInclude: vitestConfig.test.include,',
  '}));',
].join('\n');

interface ProbeResult {
  provider: string;
  include: string[];
  exclude: string[];
  thresholds: { lines: number; branches: number; functions: number; statements: number };
  testInclude: string[];
}

describe('testing-conventions vitestConfig export (e2e)', () => {
  it('resolves the base config from the package root specifier', () => {
    const out = execFileSync('node', ['--input-type=module', '-e', probe], {
      cwd: packageRoot,
      encoding: 'utf8',
    });
    const config = JSON.parse(out) as ProbeResult;

    expect(config.provider).toBe('v8');
    expect(config.include).toEqual(['src/**/*.ts']);
    expect(config.exclude).toEqual(['src/**/*.d.ts']);
    expect(config.thresholds).toEqual({
      lines: 100,
      branches: 100,
      functions: 100,
      statements: 100,
    });
    expect(config.testInclude).toEqual(['src/**/*.test.ts']);
  });
});
