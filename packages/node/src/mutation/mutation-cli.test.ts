import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { NormalizedMutant } from './to-normalized.js';

// Mock `parseArgs`, `runStryker`, `readVitestVersion`, and `fs/promises.writeFile` so the
// behaviors `mutationCLI` owns can be driven without a real mutation run. The version read is
// mocked so these tests never depend on the vitest this repo happens to install.
const { parseArgs, readVitestVersion, runStryker, writeFile } = vi.hoisted(() => ({
  parseArgs: vi.fn<(argv: string[]) => { mutate?: string[]; out?: string; testFiles?: string[] }>(),
  readVitestVersion: vi.fn<(projectRoot: string) => string | null>(),
  runStryker:
    vi.fn<(options?: { mutate?: string[]; testFiles?: string[] }) => Promise<NormalizedMutant[]>>(),
  writeFile: vi.fn<() => Promise<void>>(),
}));
vi.mock('./parse-args.js', async () => {
  const actual = await vi.importActual<typeof import('./parse-args.js')>('./parse-args.js');
  return { ...actual, parseArgs };
});
vi.mock('./read-vitest-version.js', async () => {
  const actual =
    await vi.importActual<typeof import('./read-vitest-version.js')>('./read-vitest-version.js');
  return { ...actual, readVitestVersion };
});
vi.mock('./run-stryker.js', async () => {
  const actual = await vi.importActual<typeof import('./run-stryker.js')>('./run-stryker.js');
  return { ...actual, runStryker };
});
vi.mock('node:fs/promises', async () => {
  const actual = await vi.importActual<typeof import('node:fs/promises')>('node:fs/promises');
  return { ...actual, writeFile };
});

import { mutationCLI } from './mutation-cli.js';

describe('mutationCLI', () => {
  beforeEach(() => {
    readVitestVersion.mockReturnValue('3.2.4');
  });

  afterEach(() => {
    vi.restoreAllMocks();
    parseArgs.mockReset();
    readVitestVersion.mockReset();
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

  it('passes the parsed test-file patterns through', async () => {
    parseArgs.mockReturnValue({ testFiles: ['src/**'] });
    runStryker.mockResolvedValue([]);
    const write = vi.spyOn(process.stdout, 'write').mockImplementation(() => true);

    await mutationCLI(['--test-files', 'src/**']);

    expect(runStryker).toHaveBeenCalledWith({ testFiles: ['src/**'] });
    expect(write).toHaveBeenCalledWith('[]\n');
  });

  it('propagates a failed run so the caller can map it to an exit code', async () => {
    parseArgs.mockReturnValue({});
    runStryker.mockRejectedValue(new Error('boom'));

    await expect(mutationCLI([])).rejects.toThrow('boom');
  });

  it('refuses to run at all on a vitest the runner mis-scores, rather than reporting survivors', async () => {
    parseArgs.mockReturnValue({});
    readVitestVersion.mockReturnValue('5.0.1');

    await expect(mutationCLI([])).rejects.toThrow('vitest 5.0.1');
    expect(runStryker).not.toHaveBeenCalled();
  });
});
