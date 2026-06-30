import { afterEach, describe, expect, it, vi } from 'vitest';

// The launcher's one collaborator is bin-shim's `main()`. Mock it so the behaviors bin.ts
// owns — forwarding the binary's exit code, reporting a launch failure, and injecting the TS
// mutation adapter path — can be driven without spawning a real binary. `vi.hoisted` makes
// the mock available to the hoisted `vi.mock` factory.
const { main } = vi.hoisted(() => ({ main: vi.fn() }));
vi.mock('bin-shim', () => ({ main }));

const ADAPTER_ENV = 'TESTING_CONVENTIONS_TS_MUTATION_ADAPTER';

describe('bin', () => {
  afterEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
    main.mockReset();
    delete process.env[ADAPTER_ENV];
  });

  it('forwards the binary exit code that main() resolves to', async () => {
    main.mockResolvedValue(3);
    const exit = vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);

    await import('./bin.js');
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

    await import('./bin.js');
    await new Promise((resolve) => setImmediate(resolve));

    expect(write).toHaveBeenCalledWith('boom\n');
    expect(exit).toHaveBeenCalledWith(1);
  });

  it('injects the bundled TS mutation adapter path when none is set', async () => {
    delete process.env[ADAPTER_ENV];
    main.mockResolvedValue(0);
    vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);

    await import('./bin.js');

    expect(process.env[ADAPTER_ENV]).toMatch(/mutation-cli\.js$/);
  });

  it('leaves an explicit adapter path override in place', async () => {
    process.env[ADAPTER_ENV] = '/custom/adapter.js';
    main.mockResolvedValue(0);
    vi.spyOn(process, 'exit').mockImplementation((() => undefined) as never);

    await import('./bin.js');

    expect(process.env[ADAPTER_ENV]).toBe('/custom/adapter.js');
  });
});
