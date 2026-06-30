import { describe, expect, it } from 'vitest';

import { parseArgs } from './parse-args.js';

describe('parseArgs', () => {
  it('parses --mutate (comma-separated) and --out', () => {
    expect(parseArgs(['--mutate', 'src/a.ts:2-4,src/b.ts:9', '--out', '/tmp/r.json'])).toEqual({
      mutate: ['src/a.ts:2-4', 'src/b.ts:9'],
      out: '/tmp/r.json',
    });
  });

  it('returns an empty object when neither flag is present', () => {
    expect(parseArgs([])).toEqual({});
  });

  it('treats a flag with no following value as absent', () => {
    expect(parseArgs(['--mutate', '--out'])).toEqual({ mutate: ['--out'] });
    expect(parseArgs(['--out'])).toEqual({});
  });
});
