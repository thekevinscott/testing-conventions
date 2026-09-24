import { describe, expect, it } from 'vitest';

import { readVitestVersion } from './read-vitest-version.js';

describe('readVitestVersion', () => {
  it("reports the version from the project's resolved vitest", () => {
    expect(readVitestVersion('/project', () => ({ version: '4.1.11' }))).toBe('4.1.11');
  });

  it('reports null when the project resolves no vitest', () => {
    expect(
      readVitestVersion('/project', () => {
        throw new Error('Cannot find module');
      }),
    ).toBeNull();
  });

  it('reports null when the resolved package declares no version', () => {
    expect(readVitestVersion('/project', () => ({}))).toBeNull();
  });

  it('reports null when the version is not a string', () => {
    expect(readVitestVersion('/project', () => ({ version: 5 }))).toBeNull();
  });

  it('resolves from the project root it is given', () => {
    const seen: string[] = [];

    readVitestVersion('/some/project', (root) => {
      seen.push(root);
      return { version: '3.2.4' };
    });

    expect(seen).toEqual(['/some/project']);
  });
});
