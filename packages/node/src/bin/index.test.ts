import type { MainOpts } from 'bin-shim';
import { afterEach, describe, expect, it, vi } from 'vitest';

// Mock bin-shim's `main()` so the launcher's behaviors run without spawning a real binary.
// `vi.hoisted` makes the mock available to the hoisted `vi.mock` factory.
const { main } = vi.hoisted(() => ({ main: vi.fn<(opts: MainOpts) => Promise<number>>() }));
vi.mock('bin-shim', () => ({ main }));

// The launcher reads `process.argv.slice(2)` at import time, so set argv, import a fresh module
// copy, then flush the microtask that calls `process.exit`.
async function runBin(args: string[]): Promise<void> {
  process.argv = ['node', 'index.js', ...args];
  await import('./index.js');
  await new Promise((resolve) => setImmediate(resolve));
}

describe('bin', () => {
  const realArgv = process.argv;

  afterEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
    main.mockReset();
    process.argv = realArgv;
  });

  it('forwards the binary exit code that main() resolves to', async () => {
    main.mockResolvedValue(3);
    const exit = vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);

    await runBin(['unit', 'colocated-test', '--language', 'typescript', 'src']);

    expect(main).toHaveBeenCalledWith(
      expect.objectContaining({
        scope: 'testing-conventions',
        binaryName: 'testing-conventions',
        binaryDir: '',
      }),
    );
    expect(exit).toHaveBeenCalledWith(3);
  });

  it('prints the message and exits 1 when main() rejects', async () => {
    main.mockRejectedValue(new Error('boom'));
    const exit = vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);
    const write = vi.spyOn(process.stderr, 'write').mockImplementation(() => true);

    await runBin(['workflow', 'ci.yml']);

    expect(write).toHaveBeenCalledWith('boom\n');
    expect(exit).toHaveBeenCalledWith(1);
  });

  it('appends the bundled adapter path to a `unit mutation` invocation', async () => {
    main.mockResolvedValue(0);
    vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);

    await runBin(['unit', 'mutation', '--language', 'typescript', 'src']);

    const argv = main.mock.calls[0][0].argv ?? [];
    expect(argv.slice(0, -1)).toEqual([
      'unit',
      'mutation',
      '--language',
      'typescript',
      'src',
      '--ts-mutation-adapter',
    ]);
    expect(argv[argv.length - 1]).toMatch(/mutation\/main\.js$/);
  });

  it('leaves a non-mutation invocation untouched', async () => {
    main.mockResolvedValue(0);
    vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);

    await runBin(['unit', 'coverage', '--language', 'typescript', 'src']);

    expect(main.mock.calls[0][0].argv).toEqual([
      'unit',
      'coverage',
      '--language',
      'typescript',
      'src',
    ]);
  });
});
