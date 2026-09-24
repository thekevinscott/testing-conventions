import { type Mock, describe, expect, it, vi } from 'vitest';

import { type BootstrapIo, publishBootstrapStubs } from './publish-bootstrap-stubs.js';

type FakeIo = { [K in keyof BootstrapIo]: Mock<BootstrapIo[K]> };

function fakeIo(status: number | null = 0): FakeIo {
  let dirs = 0;
  return {
    makeTempDir: vi.fn<BootstrapIo['makeTempDir']>(() => `/tmp/stub-${String(++dirs)}`),
    writeFile: vi.fn<BootstrapIo['writeFile']>(),
    run: vi.fn<BootstrapIo['run']>(() => ({ status })),
    removeDir: vi.fn<BootstrapIo['removeDir']>(),
  };
}

describe('publishBootstrapStubs', () => {
  it('publishes every name in the input from a directory of its own', () => {
    const io = fakeIo();

    publishBootstrapStubs(io, 'testing-conventions, @testing-conventions/darwin-arm64');

    expect(io.makeTempDir).toHaveBeenCalledTimes(2);
    expect(io.run.mock.calls.map(([, , options]) => options.cwd)).toEqual([
      '/tmp/stub-1',
      '/tmp/stub-2',
    ]);
  });

  it('makes each directory under the system temp root', () => {
    const io = fakeIo();

    publishBootstrapStubs(io, 'testing-conventions');

    expect(io.makeTempDir).toHaveBeenCalledWith(expect.stringContaining('bootstrap-npm-'));
  });

  it('writes the stub manifest for the name into the directory it publishes from', () => {
    const io = fakeIo();

    publishBootstrapStubs(io, 'testing-conventions');

    const [path, contents] = io.writeFile.mock.calls[0];
    expect(path).toBe('/tmp/stub-1/package.json');
    expect(JSON.parse(contents) as Record<string, unknown>).toMatchObject({
      name: 'testing-conventions',
      version: '0.0.0-bootstrap',
    });
  });

  it('tags the publish, so npm accepts a prerelease', () => {
    const io = fakeIo();

    publishBootstrapStubs(io, 'testing-conventions');

    expect(io.run).toHaveBeenCalledWith(
      'npm',
      ['publish', '--access', 'public', '--tag', 'bootstrap'],
      expect.objectContaining({ stdio: 'inherit' }),
    );
  });

  it('removes each directory once its publish has run', () => {
    const io = fakeIo();

    publishBootstrapStubs(io, 'a,b');

    expect(io.removeDir.mock.calls.map(([path]) => path)).toEqual(['/tmp/stub-1', '/tmp/stub-2']);
    expect(io.removeDir).toHaveBeenCalledWith('/tmp/stub-1', { recursive: true, force: true });
  });

  it('throws naming the package npm rejected, and still removes its directory', () => {
    const io = fakeIo(1);

    expect(() => {
      publishBootstrapStubs(io, 'a,b');
    }).toThrow('publishing the a stub failed: npm exited 1');
    expect(io.run).toHaveBeenCalledTimes(1);
    expect(io.removeDir).toHaveBeenCalledTimes(1);
  });

  it('reports a signalled npm as a failure too', () => {
    const io = fakeIo(null);

    expect(() => {
      publishBootstrapStubs(io, 'a');
    }).toThrow('publishing the a stub failed: npm exited null');
  });

  it('publishes nothing and throws when the input names no package', () => {
    const io = fakeIo();

    expect(() => {
      publishBootstrapStubs(io, '  ');
    }).toThrow('`packages` names nothing to bootstrap');
    expect(io.makeTempDir).not.toHaveBeenCalled();
    expect(io.run).not.toHaveBeenCalled();
  });
});
