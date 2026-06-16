import { afterEach, describe, expect, it, vi } from 'vitest';

// The launcher's one collaborator is bin-shim's `main()`. Mock it so the two
// behaviors bin.ts owns — forwarding the binary's exit code, and reporting a
// launch failure — can be driven without spawning a real binary. `vi.hoisted`
// makes the mock available to the hoisted `vi.mock` factory.
const { main } = vi.hoisted(() => ({ main: vi.fn() }));
vi.mock('bin-shim', () => ({ main }));

describe('bin', () => {
  afterEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
    main.mockReset();
  });

  it('forwards the binary exit code that main() resolves to', async () => {
    main.mockResolvedValue(3);
    const exit = vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);

    await import('./bin');
    await new Promise((resolve) => setImmediate(resolve));

    expect(main).toHaveBeenCalledWith(
      expect.objectContaining({
        scope: 'testing-conventions',
        binaryName: 'testing-conventions',
      }),
    );
    expect(exit).toHaveBeenCalledWith(3);
  });

  it('prints the message and exits 1 when main() rejects', async () => {
    main.mockRejectedValue(new Error('boom'));
    const exit = vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);
    const write = vi.spyOn(process.stderr, 'write').mockImplementation(() => true);

    await import('./bin');
    await new Promise((resolve) => setImmediate(resolve));

    expect(write).toHaveBeenCalledWith('boom\n');
    expect(exit).toHaveBeenCalledWith(1);
  });
});
