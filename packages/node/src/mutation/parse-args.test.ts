import { describe, expect, it } from 'vitest';

import { parseArgs } from './parse-args.js';

describe('parseArgs', () => {
  it('parses --out and --mutate (comma-split into a list)', () => {
    expect(parseArgs(['--mutate', 'src/a.ts:2-4,src/b.ts:9', '--out', '/tmp/r.json'])).toEqual({
      mutate: ['src/a.ts:2-4', 'src/b.ts:9'],
      out: '/tmp/r.json',
    });
  });

  it('parses --out on its own', () => {
    expect(parseArgs(['--out', '/tmp/r.json'])).toEqual({ out: '/tmp/r.json' });
  });

  it('parses --mutate on its own', () => {
    expect(parseArgs(['--mutate', 'src/a.ts:2'])).toEqual({ mutate: ['src/a.ts:2'] });
  });

  it('parses --vitest-dir', () => {
    expect(parseArgs(['--vitest-dir', 'src'])).toEqual({ vitestDir: 'src' });
  });

  it('returns an empty object when neither flag is present', () => {
    expect(parseArgs([])).toEqual({});
  });
});
