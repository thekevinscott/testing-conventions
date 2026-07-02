import { afterEach, describe, expect, it, vi } from 'vitest';

// main.ts's one job is to run `mutationCLI` over the process arguments and map a rejected run onto
// stderr + a non-zero exit code. Mock `mutationCLI` so both can be driven without a real run.
const { mutationCLI } = vi.hoisted(() => ({
  mutationCLI: vi.fn<(argv: string[]) => Promise<void>>(),
}));
vi.mock('./mutation-cli.js', () => ({ mutationCLI }));

// main.ts runs its work at import time, reading `process.argv.slice(2)`; set argv, import a fresh
// copy, then flush the microtask that the `.catch` runs on.
async function runMain(args: string[]): Promise<void> {
  process.argv = ['node', 'main.js', ...args];
  await import('./main.js');
  await new Promise((resolve) => setImmediate(resolve));
}

describe('main', () => {
  const realArgv = process.argv;

  afterEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
    mutationCLI.mockReset();
    process.argv = realArgv;
    process.exitCode = undefined;
  });

  it('runs mutationCLI over the process arguments', async () => {
    mutationCLI.mockResolvedValue();

    await runMain(['--out', '/tmp/r.json']);

    expect(mutationCLI).toHaveBeenCalledWith(['--out', '/tmp/r.json']);
  });

  it('prints the message and sets a failing exit code when the run rejects', async () => {
    mutationCLI.mockRejectedValue(new Error('boom'));
    const write = vi.spyOn(process.stderr, 'write').mockImplementation(() => true);

    await runMain([]);

    expect(write).toHaveBeenCalledWith('boom\n');
    expect(process.exitCode).toBe(1);
  });
});
