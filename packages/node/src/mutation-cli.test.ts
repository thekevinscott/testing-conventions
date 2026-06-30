import { afterEach, describe, expect, it, vi } from 'vitest';

import type { NormalizedMutant } from './to-normalized.js';

// The entry's collaborators are `parseArgs`, `runStryker`, and `fs.writeFileSync`; mock all so
// the behaviors this file owns — writing the JSON to a file or stdout, and reporting a failure
// — can be driven without a real mutation run. `vi.hoisted` exposes them to the hoisted mocks.
const { parseArgs, runStryker, writeFileSync } = vi.hoisted(() => ({
  parseArgs: vi.fn<(argv: string[]) => { mutate?: string[]; out?: string }>(),
  runStryker: vi.fn<(options?: { mutate?: string[] }) => Promise<NormalizedMutant[]>>(),
  writeFileSync: vi.fn(),
}));
vi.mock('./parse-args.js', () => ({ parseArgs }));
vi.mock('./run-stryker.js', () => ({ runStryker }));
vi.mock('node:fs', async (importOriginal) => ({
  ...(await importOriginal<typeof import('node:fs')>()),
  writeFileSync,
}));

describe('mutation-cli', () => {
  afterEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
    parseArgs.mockReset();
    runStryker.mockReset();
    writeFileSync.mockReset();
    process.exitCode = undefined;
  });

  it('writes the normalized JSON to the --out file, passing parsed mutate ranges through', async () => {
    const survivor: NormalizedMutant = { file: 'src/a.ts', line: 2, status: 'survived', mutator: 'X' };
    parseArgs.mockReturnValue({ mutate: ['src/a.ts:2-4'], out: '/tmp/r.json' });
    runStryker.mockResolvedValue([survivor]);

    await import('./mutation-cli.js');
    await new Promise((resolve) => setImmediate(resolve));

    expect(runStryker).toHaveBeenCalledWith({ mutate: ['src/a.ts:2-4'] });
    expect(writeFileSync).toHaveBeenCalledWith('/tmp/r.json', `${JSON.stringify([survivor])}\n`);
  });

  it('writes to stdout and runs with no mutate scope when neither flag is given', async () => {
    parseArgs.mockReturnValue({});
    runStryker.mockResolvedValue([]);
    const write = vi.spyOn(process.stdout, 'write').mockImplementation(() => true);

    await import('./mutation-cli.js');
    await new Promise((resolve) => setImmediate(resolve));

    expect(runStryker).toHaveBeenCalledWith({});
    expect(write).toHaveBeenCalledWith('[]\n');
    expect(writeFileSync).not.toHaveBeenCalled();
  });

  it('prints the message and sets a failing exit code when the run rejects', async () => {
    parseArgs.mockReturnValue({});
    runStryker.mockRejectedValue(new Error('boom'));
    const write = vi.spyOn(process.stderr, 'write').mockImplementation(() => true);

    await import('./mutation-cli.js');
    await new Promise((resolve) => setImmediate(resolve));

    expect(write).toHaveBeenCalledWith('boom\n');
    expect(process.exitCode).toBe(1);
  });
});
