import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

/** Loads `vitest/package.json` as resolved from a project root. Throws when the project has none. */
export type LoadVitestPackage = (projectRoot: string) => unknown;

/**
 * The `version` of the vitest that the project at `projectRoot` resolves, or `null` when it
 * resolves none or reports no version. Resolution runs from the project, not from this package, so
 * a consumer's own vitest is the one measured — including one hoisted to a workspace root.
 */
export function readVitestVersion(
  projectRoot: string,
  load: LoadVitestPackage = (root) =>
    createRequire(pathToFileURL(join(root, 'package.json')))('vitest/package.json'),
): string | null {
  try {
    const { version } = load(projectRoot) as { version?: unknown };
    return typeof version === 'string' ? version : null;
  } catch {
    return null;
  }
}
