import { afterEach, describe, expect, it, vi } from 'vitest';

import type { NormalizedMutant } from './to-normalized.js';

// `mutationCLI`'s collaborators are `parseArgs`, `runStryker`, and `fs/promises.writeFile`; mock
// all so the behaviors it owns — writing the JSON to a file or stdout, passing the parsed mutate
// ranges through, and propagating a failed run — can be driven without a real mutation run.
const { parseArgs, runStryker, writeFile } = vi.hoisted(() => ({
  parseArgs: vi.fn<(argv: string[]) => { mutate?: string[]; out?: string }>(),
  runStryker: vi.fn<(options?: { mutate?: string[] }) => Promise<NormalizedMutant[]>>(),
  writeFile: vi.fn<() => Promise<void>>(),
}));
vi.mock('./parse-args.js', () => ({ parseArgs }));
vi.mock('./run-stryker.js', () => ({ runStryker }));
vi.mock('node:fs/promises', () => ({ writeFile }));

import { mutationCLI } from './mutation-cli.js';

describe('mutationCLI', () => {
  afterEach(() => {
    vi.restoreAllMocks();
    parseArgs.mockReset();
    runStryker.mockReset();
    writeFile.mockReset();
  });

  it('writes the normalized JSON to the --out file, passing parsed mutate ranges through', async () => {
    const survivor: NormalizedMutant = { file: 'src/a.ts', line: 2, status: 'survived', mutator: 'X' };
    parseArgs.mockReturnValue({ mutate: ['src/a.ts:2-4'], out: '/tmp/r.json' });
    runStryker.mockResolvedValue([survivor]);

    await mutationCLI(['--mutate', 'src/a.ts:2-4', '--out', '/tmp/r.json']);

    expect(runStryker).toHaveBeenCalledWith({ mutate: ['src/a.ts:2-4'] });
    expect(writeFile).toHaveBeenCalledWith('/tmp/r.json', `${JSON.stringify([survivor])}\n`);
  });

  it('writes to stdout and runs with no mutate scope when neither flag is given', async () => {
    parseArgs.mockReturnValue({});
    runStryker.mockResolvedValue([]);
    const write = vi.spyOn(process.stdout, 'write').mockImplementation(() => true);

    await mutationCLI([]);

    expect(runStryker).toHaveBeenCalledWith({});
    expect(write).toHaveBeenCalledWith('[]\n');
    expect(writeFile).not.toHaveBeenCalled();
  });

  it('propagates a failed run so the caller can map it to an exit code', async () => {
    parseArgs.mockReturnValue({});
    runStryker.mockRejectedValue(new Error('boom'));

    await expect(mutationCLI([])).rejects.toThrow('boom');
  });
});
