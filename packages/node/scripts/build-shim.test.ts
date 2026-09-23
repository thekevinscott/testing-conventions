import { describe, expect, it, vi } from 'vitest';

import { buildShim, type Spawn } from './build-shim.js';

describe('buildShim', () => {
  it('runs both tsc invocations in order for a shim-building target', () => {
    const spawn = vi.fn<Spawn>().mockReturnValue({ status: 0 });

    const exitCode = buildShim(spawn, 'main', '/pkg');

    expect(exitCode).toBe(0);
    expect(spawn).toHaveBeenNthCalledWith(
      1,
      'npx',
      ['--no-install', 'tsc', '-b', '--clean', 'tsconfig.json'],
      expect.objectContaining({ cwd: '/pkg' }),
    );
    expect(spawn).toHaveBeenNthCalledWith(
      2,
      'npx',
      ['--no-install', 'tsc', '-p', 'tsconfig.json'],
      expect.objectContaining({ cwd: '/pkg' }),
    );
  });

  it('exits early for a per-triple target without spawning', () => {
    const spawn = vi.fn<Spawn>();

    const exitCode = buildShim(spawn, 'x86_64-unknown-linux-gnu', '/pkg');

    expect(exitCode).toBe(0);
    expect(spawn).not.toHaveBeenCalled();
  });

  it('propagates a non-zero exit code from the clean step without running the build step', () => {
    const spawn = vi.fn<Spawn>().mockReturnValue({ status: 2 });

    const exitCode = buildShim(spawn, 'main', '/pkg');

    expect(exitCode).toBe(2);
    expect(spawn).toHaveBeenCalledTimes(1);
  });

  it('propagates a non-zero exit code from the build step when the clean step succeeds', () => {
    const spawn = vi.fn<Spawn>().mockReturnValueOnce({ status: 0 }).mockReturnValueOnce({ status: 3 });

    const exitCode = buildShim(spawn, 'main', '/pkg');

    expect(exitCode).toBe(3);
    expect(spawn).toHaveBeenCalledTimes(2);
  });

  it('falls back to exit code 1 when a failing step reports a null status', () => {
    const spawn = vi.fn<Spawn>().mockReturnValue({ status: null });

    const exitCode = buildShim(spawn, 'main', '/pkg');

    expect(exitCode).toBe(1);
  });
});
